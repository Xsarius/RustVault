# Auto-Categorization Rules

Rules let you automate categorization so you don't have to manually assign a category to every transaction. Once set up, rules run automatically on every import and can be re-applied to existing transactions at any time.

## How Rules Work

A rule consists of **conditions** and **actions**. When all conditions match a transaction, the actions are applied.

### Conditions

Each condition tests a transaction field:

| Field | Operators | Example |
|-------|-----------|---------|
| Description | contains, equals, starts with, ends with | Description contains "Netflix" |
| Payee | contains, equals | Payee equals "Spotify" |
| Amount | equals, greater than, less than, between | Amount between 10 and 50 |
| Account | equals | Account equals "ING Checking" |
| Currency | equals | Currency equals "EUR" |
| Type | equals | Type equals "expense" |

Multiple conditions are combined with **AND** or **OR** logic.

### Actions

When conditions match, the rule can:

| Action | Description |
|--------|-------------|
| **Set category** | Assign a specific category |
| **Add tag** | Attach one or more tags |
| **Set payee** | Override the payee name |
| **Mark reviewed** | Automatically mark as reviewed |

## Creating a Rule

1. Navigate to **Rules** in the sidebar
2. Click **+ Add Rule**
3. Enter a name for the rule (e.g. "Streaming subscriptions")
4. Add one or more conditions
5. Add one or more actions
6. Save the rule

## Priority & Ordering

Rules are evaluated in order from top to bottom. The first matching rule wins — subsequent rules are not applied to the same transaction. Drag rules to reorder their priority.

## Testing Rules

Before saving, click **Test** to see which existing transactions would match the rule. This lets you verify the conditions are correct without modifying any data.

## Re-running Rules

Click **Re-run all rules** to apply the current ruleset to all existing transactions. This is useful after creating new rules or adjusting conditions. Only uncategorized transactions are affected by default — enable "overwrite existing" to re-categorize everything.

## Tips

- **Start broad, then refine** — a rule like "description contains AMZN" catches most Amazon purchases. Create more specific rules later as needed.
- **Use the test feature** — always test a rule before saving to avoid unexpected matches.
- **Order matters** — place more specific rules above generic ones so they take priority.
