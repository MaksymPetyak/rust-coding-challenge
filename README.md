## Rust coding challenge
Implements a toy engine to process transactions from csv.
Run with

```shell
cargo run -- file.path
```

## Structure
```
├── account.rs      # handles deposit, withdraw, etc. operations on client account  
├── engine.rs       # engine to process transactions line by line
├── transaction.rs  # types for transactions with serde deserialisation rules
└── main.rs         # reads csv file, passes lines through transaction engine and writes the state of accounts
```

## Criteria
Here is how I addressed different scoring criteria:
* **Basics** - cargo test/run/build should run. Used cargo fmt and clippy for formatting.
* **Completeness** - attempted to support all the mentioned transactions. 
  * deposit/withdraw/dispute/resolve/chargeback.
  * One interesting case not covered here is what happens with a withdrawal that happened between deposit and the dispute of that deposit, such that after dispute there is actually not enough funds for the withdrawal that has already happened.
  * See [account.rs](src/account.rs) for some comments and assumptions.
* **Correctness** - see unit tests in [account.rs](src/account.rs) + there some test files you can try out under [assets](/assets)
* **Safety and Robustness** - mostly has just panics, but I put TODOs for where I think should be result types and logging
* **Efficiency** - probably the most lacking aspect. Currently, would likely fail 
for very large files due to storing transaction history and wouldn't be as quick 
as solutions processing rows in parallel. I put down some extension ideas that I 
would consider given more time to address this.
* **Maintanability** - split into account/engine/transaction + used trait for client account for easier substitution in transaction engine.


## Extension ideas
Current limitations are:
* To dispute transactions you need to remember what has actually happened in the previous transaction
(unless we do more than 1 pass through the transaction file) which is why I have a 
hashmap for storing the effect of transaction on available funds. It would grow 
out of proportions for huge files.
* No parallelism.

Hence, some ideas for improvement would be
* Go through transaction in reverse chronological order - if we 
went through transactions in reverse chronological order would only need to store 
ids of transaction in dispute/resolve/chargeback, and then when we hit the 
transaction with the right id apply it as disputed/resolved/chargebacked. 
Could then avoid storing transaction log as it is done currently.
* Add parallelism - for example, do 2 passes through the 
file, first an equivalent of groupby to group transaction for a single user together,
then could spawn different processes for each user that could write output 
independently.
