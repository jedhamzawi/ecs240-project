# Impure Functions by Declaration

A "pure" function is a function that has no side-effects -- it takes an input and produces an output without modifying state.
An impure function can mutate state. The idea here is that if you want to do something "impure," you *must* declare it, and
it bleeds into calling functions and modules. 

## Overview

You have to declare a side effect occurring; if a function is *not* pure, you must say it is. We would need to detect whether
a side effect is happening and enforce it. Functions are pure by default. If a function does not return anything, it is
automatically not pure.

```
var z;

fn pure_function(x, y) {
    x + y
}

fn impure_function(x, y) {
    // sets a global variable! Has no return value.
    // this would yield a compiler error
    z = x + y;
}
```

Everything is *pure* by default; modules, etc. But they require that *everything* is pure. Once you have one thing that is *not* pure,
the rest cannot be pure.

> What are the implications then? Why is this helpful?

It's not super helpful if impure will leak to everything. The second a program is impure, none of the rest is pure.

It *would* be helpful though if you *must declare the state that is changed*. Then when you call a non-pure function, you can point to
exactly the data that will change just by "mousing over" the function (that's a stretch goal).

```
var z;

fn pure_function(x, y) {
    x + y
}

// specify the data whose state will change
fn impure(z) impure_function(x, y) {
    z = x.value + y.value;
}

// You have to be exact on what will mutate! Not just the variable,
// but the field as well (x.value), otherwise you get a compile error
// (Error: x not the most specific state change; try 'x.value')
fn impure(x) impure_function_2(x) {
    // mutates x
    x.value = 1;
}

fn impure(x.value, y.value) impure_function_3(x, y) {
    // mutates x
    x.value = 1;
    y.value = 1;
}


fn impure(x.value) impure_function_3(x, y) {
    // even with an if statement, you must declare that state
    // may change
    if y.value == 1 {
	    x.value = 0;
    }
}

// you can look at the program see a full list
// of all the things that could change state
fn impure program() {
    var x;
    var y;
    x.value = 1;
    y.value = 2;
    // the idea is when you "mouse over" or ask some static analyzer
    // about this impure_function, you'll get a specific list of things
    // whose state will change:
    //
    // mutated: x.value, y.value
	impure_function_3(x, y);
	// mutated: z
	impure_function(x, y);
}
```

## Risks

- Tracking state might be untractable
	- But it might be easier if each impure function must declare the state that changes
	- How do we enforce that though? How do we *know* when state changes? Just the assignment `=` operator? That could be enough
- For the struct idea (which I think is important to model) we would need an idea of "fields"
	- we don't need strict types per-se; we could just model fields as a `HashMap` and return `None` or some default value if it
  isn't found
- Impact? Is this helpful?
	- I think so; many bugs derive strictly due to side-effects. Having a full enumeration of everything that may mutate in an
  impure function is helpful I think. And you could go beyond enumeration possibly...
- Does this exist already?
	- ???
	- Probably. Functional languages do not allow for impure functions. Rust requires specifying when something is mutable.
  There's probably a language that requires declaring when something is impure. I need to look into it.

