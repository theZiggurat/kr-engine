# kr-engine

Toy payments engine which processes transaction csv files to generate client data.

Total *code* development time was around 3h15m, but that does not include planning and documentation. 

# Run
```
cargo run -- path/to/file.csv > accounts.csv
```

# Approach

The entire application state is held in the `Context` struct. At the start of the program, the input csv is deserialized to `Vec<Transaction>` which is moved to the `Context`. On invoking `Context::process`, use the transaction at `transactions[batch_index]` to apply a transformation to the `Client` map. 

Since deposits can be disputed, there needs to be an efficient way to find previously processed deposits by `tx` (transaction id). The `transaction_cache` maps `tx` to an index to `transactions`. Additionally, dispute, resolve, and chargeback operations need to mutate the transaction status. For instance, a transaction with the `ChargedBack` status will never be considered for any further dispute, resolve, or chargeback.

`Batch` processes all transactions starting at `batch_index` to the end of `transactions`. I went for this approach due to its flexibility and ease of reasoning. With minor tweaks to the `Context` api, you could keep pushing to the `transactions` list and run `batch` in a loop to serve a real-time context.

The other three files, `client.rs`, `errors.rs`, and `transactions.rs` are mainly type declarations and serde stuff. 

# Notes

## Assumptions
* Dispute after withdrawal - I went out on a limb and assumed that shouldn't be allowed to happen, since the user would owe us money if a chargeback were to occur. Disputes will only be applied if the target transaction amount of the dispute is less than or equal to the client's available balance.
* Disputing withdrawals - This could be my common sense failing me, but Im assuming disputing a withdrawal doesnt really happen? Because I've commited to that, only despoits are cached for later mutation and considered for disputes. 
* Withdrawals from locked accounts - This is a one line replacement, but I opted to allow locked accounts to withdrawal from their available balance. My reasoning is if the requested funds are not being held, the user is elligable to withdrawal it at any time. 

## Improvements

Given the time constaints, there are several improvements that can be made with development time.

 * **More flexible constructor** - With a Read trait based API instead of using direct file paths. This would allow the context to be used with sockets and any other struct that impls Read. 
 * **Fixed point numbers** - Given the precicion requirement of 4 decimal points, a smaller footprint fixed point number could be used in place of the `f64` in use now.
 * **Testing** - Currently there are three manually verified test cases in `./tests`. Ideally, there would be a mix of rust level unit testing and execution level integration tests ran by a script or other automation solution. 