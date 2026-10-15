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
cli-help-mask = Mask pattern, for example ?l?l?l?l?d?d (?l lowercase, ?u uppercase, ?d digit, ?s symbol, ?a any, ?? a literal question mark)
cli-help-rules = Mangling rules applied to every candidate. Comma separates alternatives, plus chains rules together: none, capitalize, upper, lower, reverse, leet, append-digits, prepend-digits, append-symbols. Digits take an optional maximum. Example: capitalize+append-digits:4,none
cli-help-threads = Worker threads, 0 means one per CPU
cli-help-lang = Interface language: en or id
cli-help-json = Emit one JSON object per line instead of a report
cli-help-quiet = Plain output without colour and without a progress bar
cli-help-resume = Continue a previous run of the same attack on the same file
cli-heading-usage = Usage:
cli-heading-options = Options:
cli-help-help = Print this help
cli-help-version = Print the version

# Report
cli-target = Target
cli-method = Method
cli-method-dictionary = wordlist
cli-method-brute-force = brute force
cli-method-mask = mask
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
log-session-resumed = Resuming from { $position } of { $total } candidates
log-session-absent = No saved session for this file, starting from the beginning
log-level-info = INFO
log-level-warn = WARN
log-level-error = ERROR
log-level-success = OK

# Progress
progress-line = { $tried } of { $total } tried, { $speed } per second, ETA { $eta }

# Window
app-window-title = Zydra — password recovery
app-subtitle = RECOVERY CORE

# Actions
action-start = START DECRYPTION
action-stop = STOP
action-reset = NEW RUN
action-browse = Browse
action-copy = Copy password
action-copied = Copied
action-options = Options

# Sidebar
sidebar-target = Target file
sidebar-target-empty = No file selected
sidebar-method = Method
sidebar-method-dictionary = Dictionary
sidebar-method-brute-force = Brute force
sidebar-wordlist = Wordlist
sidebar-wordlist-empty = No wordlist selected
sidebar-charset = Character set
charset-lowercase = Lowercase a–z
charset-uppercase = Uppercase A–Z
charset-digits = Digits 0–9
charset-symbols = Symbols !#%
charset-space = Space
sidebar-min-length = Min length
sidebar-max-length = Max length
sidebar-combinations = Combinations

# Workspace
workspace-status-idle = Ready
workspace-status-analyzing = Analyzing…
workspace-status-running = Decrypting…
workspace-status-done = Finished
workspace-status-failed = Failed
workspace-status-cancelled = Stopped
workspace-progress = Progress
workspace-activity = Activity
log-panel-empty = No activity yet
stat-elapsed = elapsed
