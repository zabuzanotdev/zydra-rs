# Zydra — katalog Bahasa Indonesia.
#
# Setiap kalimat yang dilihat pengguna tinggal di sini. Kode hanya menyebut kunci, jadi
# mengubah kalimat tidak pernah menyentuh logika, dan menambah bahasa cukup menambah satu
# berkas. Kunci harus sama persis dengan en.ftl; kunci yang hilang jatuh ke Bahasa Inggris.

app-name = Zydra
app-tagline = Pemulihan password untuk berkas terenkripsi

# Bantuan baris perintah
cli-about = Memulihkan password sebuah berkas terenkripsi
cli-help-file = Berkas target: ZIP, RAR, PDF, atau berkas shadow Unix
cli-help-wordlist = Berkas wordlist, satu kandidat per baris
cli-help-charset = Kelas karakter, dipisah koma: lowercase, uppercase, letters, digits, symbols, space
cli-help-min-length = Panjang password minimum untuk brute force
cli-help-max-length = Panjang password maksimum untuk brute force
cli-help-mask = Pola mask, misalnya ?l?l?l?l?d?d (?l huruf kecil, ?u huruf besar, ?d angka, ?s simbol, ?a semua, ?? tanda tanya biasa)
cli-help-rules = Aturan mangling yang diterapkan pada setiap kandidat. Koma memisahkan alternatif, tanda tambah merantai aturan: none, capitalize, upper, lower, reverse, leet, append-digits, prepend-digits, append-symbols. Aturan angka menerima batas opsional. Contoh: capitalize+append-digits:4,none
cli-help-threads = Jumlah worker, 0 berarti satu per CPU
cli-help-lang = Bahasa antarmuka: en atau id
cli-help-json = Keluarkan satu objek JSON per baris, bukan laporan
cli-help-quiet = Keluaran polos tanpa warna dan tanpa bilah progres
cli-help-resume = Lanjutkan serangan yang sama pada berkas yang sama dari posisi terakhir
cli-heading-usage = Penggunaan:
cli-heading-options = Pilihan:
cli-help-help = Tampilkan bantuan ini
cli-help-version = Tampilkan versi

# Laporan
cli-target = Target
cli-method = Metode
cli-method-dictionary = wordlist
cli-method-brute-force = brute force
cli-method-mask = mask
cli-candidates = Kandidat
cli-start-time = Waktu mulai
cli-end-time = Waktu selesai
cli-elapsed = Durasi

# Statistik
stat-speed = kecepatan
stat-eta = sisa waktu
stat-tried = dicoba
stat-value-unknown = tidak diketahui

# Hasil
result-granted = Password ditemukan
result-denied = Password tidak ditemukan
result-key = Password
result-detail = Cocok dengan
result-truncated = verifikasi berhenti di batas baca

# Kegagalan
error-no-match = Tidak ada kandidat yang cocok.
error-unsupported-format = Jenis berkas tidak didukung: { $name }
error-empty-wordlist = Wordlist tidak memuat satu kandidat pun.
error-file-unreadable = Berkas target tidak bisa dibaca.
error-no-charset = Pilih minimal satu kelas karakter.
error-bad-range = Rentang panjang tidak sah: minimum { $min }, maksimum { $max }, maksimum yang diizinkan { $allowed_max }.
error-invalid-mask = Pola mask tidak sah: { $pattern }
error-invalid-rule = Aturan tidak dikenal: { $rule }
error-too-many-combinations = Terlalu banyak kombinasi: { $total }, batas { $limit }.
error-no-candidate = Tidak ada yang bisa dicoba.
error-not-encrypted = Berkas ini tidak terenkripsi, jadi tidak ada password yang perlu dipulihkan.
error-backend-unavailable = Backend { $backend } tidak tersedia di build ini.
error-session-mismatch = Sesi tersimpan tidak cocok dengan berkas ini.
error-unknown = Operasi gagal.

# Masalah pemakaian, dilaporkan sebelum pekerjaan dimulai
usage-missing-target = Pilih berkas target.
usage-missing-method = Pilih sebuah wordlist atau himpunan kelas karakter untuk brute force.
usage-bad-charset = Kelas karakter tidak dikenal: { $name }
usage-bad-language = Bahasa tidak dikenal: { $value }
usage-bad-length = Minimum dan maksimum harus berupa angka, dan minimum tidak boleh melebihi maksimum.
usage-conflicting-methods = Pilih salah satu: wordlist atau brute force, jangan keduanya.

# Log
log-wordlist-loaded = Memuat { $count } kandidat dari wordlist
log-brute-force-space = Menelusuri { $count } kombinasi
log-target-selected = Target: { $name }
log-backend-missing = Backend { $backend } tidak tersedia
log-session-resumed = Melanjutkan dari { $position } dari { $total } kandidat
log-session-absent = Tidak ada sesi tersimpan untuk berkas ini, dimulai dari awal
log-level-info = INFO
log-level-warn = PERINGATAN
log-level-error = GALAT
log-level-success = BERHASIL

# Progres
progress-line = { $tried } dari { $total } dicoba, { $speed } per detik, sisa waktu { $eta }

# Jendela
app-window-title = Zydra — pemulihan password
app-subtitle = INTI PEMULIHAN

# Aksi
action-start = MULAI DEKRIPSI
action-stop = HENTIKAN
action-reset = MULAI ULANG
action-resume = LANJUTKAN
action-browse = Pilih
action-copy = Salin password
action-copied = Tersalin
action-options = Opsi

# Panel samping
sidebar-target = Berkas target
sidebar-target-empty = Belum ada berkas
sidebar-method = Metode
sidebar-method-dictionary = Kamus
sidebar-method-brute-force = Brute force
sidebar-method-mask = Mask
sidebar-wordlist = Wordlist
sidebar-wordlist-empty = Belum ada wordlist
sidebar-charset = Kelas karakter
charset-lowercase = Huruf kecil a–z
charset-uppercase = Huruf besar A–Z
charset-digits = Angka 0–9
charset-symbols = Simbol !#%
charset-space = Spasi
sidebar-min-length = Panjang min
sidebar-max-length = Panjang maks
sidebar-combinations = Kombinasi

# Mask dan aturan
sidebar-mask = Pola mask
sidebar-mask-hint = misalnya ?l?l?l?l?d?d
sidebar-mask-empty = Isi pola mask
sidebar-rules = Aturan mangling
sidebar-rules-enable = Terapkan aturan pada setiap kandidat
sidebar-rules-hint = misalnya none,capitalize+append-digits:9

# Area kerja
workspace-status-idle = Siap
workspace-status-analyzing = Menganalisis…
workspace-status-running = Mendekripsi…
workspace-status-done = Selesai
workspace-status-failed = Gagal
workspace-status-cancelled = Dihentikan
workspace-progress = Kemajuan
workspace-activity = Aktivitas
log-panel-empty = Belum ada aktivitas
stat-elapsed = durasi

# Nama nilai yang muncul di bantuan, misalnya <FILE> pada "-f, --file <FILE>"
cli-value-file = BERKAS
cli-value-wordlist = WORDLIST
cli-value-classes = KELAS
cli-value-length = PANJANG
cli-value-mask = POLA
cli-value-rules = DAFTAR
cli-value-threads = JUMLAH
cli-value-lang = KODE
# Ekspor log
log-export-header = Catatan aktivitas Zydra, diekspor pada { $time } UTC
cli-help-export-log = Tulis catatan aktivitas ke berkas setelah serangan selesai
cli-value-log = BERKAS
error-log-export = Catatan aktivitas tidak bisa ditulis
action-export-log = Ekspor log
log-exported = Catatan aktivitas ditulis ke { $path }
