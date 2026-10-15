# Zydra — English catalogue.
#
# Every user-visible sentence lives here. The code only ever refers to keys, so a wording
# change never requires touching logic, and adding a language means adding one file.

app-name = Zydra
app-tagline = Password recovery for encrypted files

# Command line help
cli-about = Recover the password of an encrypted file
cli-help-file = Target file: ZIP, RAR, PDF, or a Unix shadow file
cli-help-wordlist = Wordlist file with one candidate per line
cli-help-charset = Character classes, comma separated: lowercase, uppercase, letters, digits, symbols, space
cli-help-min-length = Minimum password length for brute force
cli-help-max-length = Maximum password length for brute force
cli-help-threads = Worker threads, 0 means one per CPU
cli-help-lang = Interface language: en or id
cli-help-json = Emit one JSON object per line instead of a report
cli-help-quiet = Plain output without colour and without a progress bar
cli-heading-usage = Usage:
cli-heading-options = Options:
cli-help-help = Print this help
cli-help-version = Print the version

# Report
cli-target = Target
cli-method = Method
cli-method-dictionary = wordlist
cli-method-brute-force = brute force
cli-candidates = Candidates
cli-start-time = Start time
cli-end-time = End time
cli-elapsed = Elapsed

# Statistics
stat-speed = speed
stat-eta = ETA
stat-tried = tried
stat-value-unknown = n/a

# Result
result-granted = Password found
result-denied = Password not found
result-key = Password
result-detail = Match
result-truncated = verification stopped at the read limit

# Failures
error-no-match = No candidate matched.
error-unsupported-format = Unsupported file type: { $name }
error-empty-wordlist = The wordlist has no candidates.
error-file-unreadable = The target file cannot be read.
error-no-charset = Select at least one character class.
error-bad-range = Invalid length range: minimum { $min }, maximum { $max }, allowed maximum { $allowed_max }.
error-invalid-mask = Invalid mask pattern: { $pattern }
error-invalid-rule = Unknown rule: { $rule }
error-too-many-combinations = Too many combinations: { $total }, limit { $limit }.
error-no-candidate = Nothing to try.
error-backend-unavailable = The { $backend } backend is not available in this build.
error-session-mismatch = The saved session does not match this file.
error-unknown = The operation failed.

# Usage problems, reported before any work starts
usage-missing-target = Choose a target file.
usage-missing-method = Choose a wordlist or a brute force character set.
usage-bad-charset = Unknown character class: { $name }
usage-bad-language = Unknown language: { $value }
usage-bad-length = Minimum and maximum must be numbers, and minimum must not exceed maximum.
usage-conflicting-methods = Choose either a wordlist or a brute force, not both.

# Log
log-wordlist-loaded = Loaded { $count } candidates from the wordlist
log-brute-force-space = Searching { $count } combinations
log-target-selected = Target: { $name }
log-backend-missing = Backend { $backend } is not available
log-level-info = INFO
log-level-warn = WARN
log-level-error = ERROR
log-level-success = OK

# Progress
progress-line = { $tried } of { $total } tried, { $speed } per second, ETA { $eta }
