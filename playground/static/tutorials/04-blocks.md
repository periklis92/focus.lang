So far we've written only single line function. But in `focus` you can use indentation to write multiline functions and blocks.

There are two types of statements that expect a block in `focus`. The `if` statement and function definitions. The last expression in a block is also the return value of the block.

```focus
let result = if true then 
    1
else
    2
```
Here the value of `result` is 1.

```focus
let add a b = a + b
```
The function performs the addition and return the result.

## Multiline blocks
```focus
let process_data data = 
    if data == () then
        Io.print "Invalid data"
        ()
    else
        Io.print "Processing data..."
        # do more processing
        result # return processed data
```
