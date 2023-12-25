Variables are declared with the keyword `let` followed by a name and they are containers for values.

```focus
let num = 2
```

## Values
There are 7 main types of values in `focus`.

- Unit `()`: This is the simplest of the types and is used in some cases to denote the lack of a real value.
- Boolean `true`/`false`
- Integer `1`, `2`...: A whole number
- Number `1.1`, `1.2`: A floating point number
- Array `[...]`: An ordered collection of values
- Table `{...}`: An assosiative array
- Function `fn <args> -> <code>`: An executable piece of code

Functions are a first class citizen in focus so they can be stored and accesed through variables. We'll talk more about them in the next chapter.

## Arrays
Arrays in `focus` are initialized using the square brackets `[]` and they can contain different types of values.
```focus
let empty_array = []
let simple_array = [1, 2, 3]
let different_types = [1, true, empty_array, simple_array] # [1, true, [], [1, 2, 3]]
```

## Tables
Tables are initialized using the curly braces `{}`. The keys can be a string or any other value.

```focus
let table = {
    "true": 1, # Here we use the quotes because 'true' is a keyword and we want to create a string key
    [true]: 2, # Use the square brackets to use the boolean true as a key
    another_key: 3 # A string type key
}

table.true # Not valid. true is a keyword
table["true"] # Valid. 1
table[true] # 2
table.another_key # 3
table["another_key"] # 3
```

Try to create a few variables and display them using the `Io.print` function.