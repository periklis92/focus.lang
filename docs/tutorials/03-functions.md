Functions in `focus` can be declared the same way as variables. The key difference is that functions take at least one argument and return one value when called.

This is the simplest function that can be declared.

```focus
let func () = ()
```

It can be called with a unit argument and returns a unit.

Try to write a function called `add` that takes two arguments and 
returns their sum.

```focus
func () # returns ()
```

Functions can be recursive when declared this way.
```focus
let recursive a = if a <= 0 then "done" else recursive (a - 1)
```

## Anonymous functions
Anonymous functions are usefull when they need to be passed as an argument to another function.

They are declared using the keyword `fn` followed by the arguments (or unit `()` is the case of no arguments) followed by the arrow `->` and the code.

This piece of code takes the array `[1, 2, 3]` and applies the function that is passed as the second argument to that array.
```focus
Iter.map [1, 2, 3] fn a -> a + 1
```

## Function calls
Function calls, in general, can be written as `<function to be called>` `<arguments or ()>`.

The pipe operator `|>` can be used to pipe the result of one call, as the **first** argument of the next. This can be usefull when, for example, we want to process an array by applying multiple `Iter` functions.

For example this:
```focus
let result = Iter.map [1, 2, 3] fn a -> a + 1 # [2, 3, 4]
let final_result = Iter.filter result fn a -> if a > 3 then a else () # [4]
```

Can be rewritten like this:
```focus
let result = Iter.map [1, 2, 3] fn a -> a + 1 |> Iter.filter fn a -> if a > 3 then a else () # [4]
```