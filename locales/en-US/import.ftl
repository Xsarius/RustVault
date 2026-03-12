# Import messages
# Project Fluent format — https://projectfluent.org/

# Upload
import-upload-success = File uploaded successfully.
import-file-too-large = File exceeds the maximum size of { $max }.
import-extension-not-allowed = File extension "{ $ext }" is not allowed.

# Format detection
import-format-detected = Detected format: { $format }.
import-format-unsupported = Unsupported file format.
import-format-detection-failed = Could not determine the file format automatically.

# Parsing
import-parse-success = Parsed { $count } transactions.
import-parse-failed = Failed to parse file: { $details }.
import-parse-no-transactions = No transactions found in the file.

# Column mapping
import-mapping-required = Column mapping is required for this format.
import-mapping-missing-field = Required field "{ $field }" is not mapped.

# Execution
import-started = Import started.
import-complete = Successfully imported { $count } transactions.
import-rolled-back = Import rolled back. No transactions were saved.
import-partial-failure = Imported { $success } of { $total } transactions. { $failed } failed.

# Rules
import-rules-applied = Auto-categorization rules applied to { $count } transactions.

# Duplicates
import-duplicates-found = { $count } potential duplicates detected.
import-duplicates-skipped = Skipped { $count } duplicate transactions.

# Validation
import-file-validation-failed = File validation failed: { $details }.
