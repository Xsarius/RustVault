# Transaction messages
# Project Fluent format — https://projectfluent.org/

# CRUD
transaction-created = Transaction created successfully.
transaction-updated = Transaction updated successfully.
transaction-deleted = Transaction deleted successfully.
transaction-not-found = Transaction { $id } was not found.

# Bulk operations
transactions-bulk-categorized = { $count } transactions categorized.
transactions-bulk-tagged = { $count } transactions tagged.
transactions-bulk-deleted = { $count } transactions deleted.
transactions-bulk-reviewed = { $count } transactions marked as reviewed.

# Validation
transaction-amount-required = Transaction amount is required.
transaction-date-required = Transaction date is required.
transaction-account-required = An account must be selected.
transaction-description-too-long = Description must be at most { $max } characters.
transaction-invalid-type = Invalid transaction type: { $type }.

# Transfers
transfer-detected = Transfer detected between { $from } and { $to }.
transfer-same-account = Source and destination accounts must be different.

# Duplicates
duplicate-warning = This transaction may be a duplicate of an existing entry.
