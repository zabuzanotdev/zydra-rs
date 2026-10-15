#!/usr/bin/env python3
"""Membuat fixture PDF terenkripsi yang konform untuk pengujian Zydra.

# Kenapa berkas ini ada

Fixture-nya harus dibuat dengan cara yang bisa dipertanggungjawabkan, bukan sekadar
berkas biner tanpa asal-usul. Skrip ini mendokumentasikan tepat bagaimana fixture itu
dibuat, dan bisa dijalankan ulang kapan saja.

# Kenapa tidak memakai pustaka yang sudah ada

PyMuPDF tersedia di mesin pengembangan dan bisa membuat PDF terenkripsi, tetapi menulis
kamus /Encrypt langsung di dalam trailer. Spesifikasi PDF mewajibkannya berupa
**referensi tidak langsung**, dan `lopdf` menegakkan hal itu: berkas keluaran PyMuPDF
terbaca oleh MuPDF tetapi tidak dikenali sebagai terenkripsi oleh `lopdf`.

Membuat sendiri berarti fixture-nya sesuai spesifikasi, dan pengujiannya menguji jalur
yang memang dipakai berkas di dunia nyata. Algoritmanya dari ISO 32000-1:
revisi 2 (RC4 40 bit), yaitu Algorithm 2 untuk kunci, Algorithm 3 untuk O, dan
Algorithm 4 untuk U.

Jalankan:  python make_encrypted_pdf.py
"""

from __future__ import annotations

import hashlib
import pathlib

PASSWORD = "kr4k3n_1987"
OUTPUT = pathlib.Path(__file__).with_name("encrypted.pdf")

# Tabel padding standar dari spesifikasi.
PADDING = bytes(
    [
        0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41,
        0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
        0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80,
        0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
    ]
)


def rc4(key: bytes, data: bytes) -> bytes:
    """RC4. Pendek, dan menulisnya sendiri menghindari ketergantungan tambahan."""
    state = list(range(256))
    j = 0
    for i in range(256):
        j = (j + state[i] + key[i % len(key)]) & 0xFF
        state[i], state[j] = state[j], state[i]

    out = bytearray()
    i = j = 0
    for byte in data:
        i = (i + 1) & 0xFF
        j = (j + state[i]) & 0xFF
        state[i], state[j] = state[j], state[i]
        out.append(byte ^ state[(state[i] + state[j]) & 0xFF])
    return bytes(out)


def pad(password: bytes) -> bytes:
    """Padding password menjadi tepat 32 byte sesuai spesifikasi."""
    password = password[:32]
    return password + PADDING[: 32 - len(password)]


def object_key(encryption_key: bytes, number: int, generation: int = 0) -> bytes:
    """Kunci per objek: MD5(kunci + nomor + generasi), dipotong 10 byte untuk RC4 40 bit."""
    material = (
        encryption_key
        + number.to_bytes(3, "little")
        + generation.to_bytes(2, "little")
    )
    return hashlib.md5(material).digest()[:10]


def build() -> bytes:
    user = pad(PASSWORD.encode())
    owner = pad(PASSWORD.encode())  # tanpa owner password terpisah
    permissions = -1  # semua izin

    # Elemen pertama array /ID berkas ikut masuk ke dalam turunan kunci. Nilainya harus
    # sama dengan yang ditulis di trailer, jadi ditentukan lebih dulu dan dipakai di kedua
    # tempat. Nilai tetap dipakai supaya fixture bisa dibuat ulang dengan hasil yang sama.
    file_id = hashlib.md5(b"zydra-fixture").digest()

    owner_key = hashlib.md5(owner).digest()[:5]
    o_value = rc4(owner_key, user)

    encryption_key = hashlib.md5(
        user
        + o_value
        + (permissions & 0xFFFFFFFF).to_bytes(4, "little")
        + file_id
    ).digest()[:5]

    u_value = rc4(encryption_key, PADDING)

    def encrypt(number: int, data: bytes) -> bytes:
        return rc4(object_key(encryption_key, number), data)

    content = b"BT /F1 12 Tf 20 120 Td (Zydra test fixture) Tj ET"
    encrypted_content = encrypt(4, content)

    objects: list[bytes] = [
        b"<</Type/Catalog/Pages 2 0 R>>",
        b"<</Type/Pages/Kids[3 0 R]/Count 1>>",
        b"<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]"
        b"/Resources<</Font<</F1 5 0 R>>>>/Contents 4 0 R>>",
        b"<</Length %d>>\nstream\n%s\nendstream" % (len(encrypted_content), encrypted_content),
        b"<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>",
        # Kamus enkripsi. Ditulis sebagai objek tersendiri, dan trailer menunjuk ke sini
        # lewat referensi tidak langsung seperti yang diwajibkan spesifikasi.
        b"<</Filter/Standard/V 1/R 2/O <%s>/U <%s>/P %d>>"
        % (o_value.hex().encode(), u_value.hex().encode(), permissions),
    ]

    out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets: list[int] = []
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % number + body + b"\nendobj\n"

    xref_offset = len(out)
    count = len(objects) + 1
    out += b"xref\n0 %d\n" % count
    out += b"0000000000 65535 f \n"
    for offset in offsets:
        out += b"%010d 00000 n \n" % offset

    file_id_hex = file_id.hex().encode()
    out += b"trailer\n<</Size %d/Root 1 0 R/Encrypt 6 0 R/ID[<%s><%s>]>>\n" % (
        count,
        file_id_hex,
        file_id_hex,
    )
    out += b"startxref\n%d\n%%%%EOF\n" % xref_offset
    return bytes(out)


def main() -> None:
    data = build()
    OUTPUT.write_bytes(data)
    print(f"menulis {OUTPUT} ({len(data)} byte)")
    print(f"password: {PASSWORD}")


if __name__ == "__main__":
    main()
