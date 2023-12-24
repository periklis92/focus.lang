
This is a simple "Hello World" example.

## The `main` function

Every program's entry point is the `main` function.

In order to create a function in `focus-lang` you need to use the `let` keyword, which is used to declare a name, both for variables
and functions, followed by any number of arguments. All functions need to take at least one argument and return one value. However the `main`
function is a function with side effects, that takes no real arguments. This is why we use `()` as its only argument.

```focus
let main () = Io.print "Hello World"
```

## Using `modules`

In order to print "Hello World" to the console, we need to use the built-in module `Io`. Modules are organizational units that contain functions and/or values.

Here we call the function `print`.

Try to `Run` *(button in the top right or `F5`)* the program and see the output in the console below.

## The `print` function

The print function is a built-in function that can take any number of arguments and print them in the output in order.


Try to split `"Hello"` and `"World"` into two separate strings and run the program again. Don't forget to leave an empty space between them.