# Uindex - Universal index

[Uindex][0] is a data store for linear data with rich structure.
With "linear data with rich structure",
we mean any data that can be parsed with some [parsing expression grammar][1] (PEG).
There is no need to transform the data to store it with uindex:
we can describe the structure of our data with a PEG, provide it to uindex,
and then enter the data straight into uindex,
as sentences structured according to the top production of the PEG.
If we think of uindex as a database, we might think of the PEG as the schema of the db.

Queries to uindex also take the forms specified in the PEG:
they are just sentences in the language definded by the PEG.
In addition, in queries we can use variables in place of any of the productions in the PEG,
to retrieve unknown values from the index.
We can also use more than a single sentence in the queries.

Uindex stores that data in such a manner that
adding new sentences to the index is O(1) wrt the size of the index,
and also that allows queries to be resolved in O(1) wrt the size of the index.

## Example

As an example, we will build a very simple database of triples, subject-verb-object.
Words (be they subjects, verbs or objects) will have the form of strings of alphanumeric characters,
and sentences will consist in 3 such words separated by spaces and terminated by a dot.
So an example sentence in this db could be ``susan likes oranges.``.
The PEG for this would be:

```pest
fact        = { word ~ word ~ word }

word        = { ASCII_ALPHANUMERIC+ }

WHITESPACE  = { " " | "\t" | "\r" | "\n" }
```
&nbsp;
&nbsp;

Uindex uses [Pest][2] to deal with PEGs, so look up the pest documentation for the [specific
syntax][4] for PEGs used by uindex. In particular, the ``WHITESPACE`` rule is special for Pest,
and if provided, it will be inserted between any 2 other chained productions.

The only uindex specific thing in the grammar above is naming the top production ``fact``;
uindex requires it.

Now we want to specify which of our productions can be unknowns in our queries.
We transform our grammar as follows:

```pest
var         = @{ "X" ~ ('0'..'9')+ }

fact        = { word ~ word ~ word }

v_word      = @{ ASCII_ALPHANUMERIC+ }

word        = _{ var | v_word }

WHITESPACE  = { " " | "\t" | "\r" | "\n" }
```
&nbsp;
&nbsp;

In short, we provide a production for variables, we prefix the production we want variables to be able to match
with ``v_``, and then we provide a new production, with the old name (without the ``v_`` prefix),
in which we mix the old production with `var`.
We can mark as many productions like this as we want, and they can be terminal or not.

To use this grammar, we need to set up some boilerplate. At this moment, uindex can only be used from [Rust][3].

So we store the code above in a file named ``grammar.pest``, which we place at the root of our rust package.

But first of all, we must add some dependencies to our `Cargo.toml`:

```toml
[dependencies]
uindex = "0.1.1"
uindex_derive = "0.1.1"
pest = "2.1.3"
pest_derive = "2.1.0"
log = "0.4"
env_logger = "0.7.1"
```
&nbsp;
&nbsp;

Then, we build our knowledge base based on the grammar, adding this code in a rust module:

```rust
use crate::uindex::kbase::DBGen;
use crate::uindex::kbase::DataBase;

extern crate uindex
#[macro_use]
extern crate uindex_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(DBGen)]
#[grammar = "grammar.pest"]
pub struct DBGenerator;
```
&nbsp;
&nbsp;

This provides us with a ``struct`` ``DBGenerator``, whose only responsibility is to
create databases that can hold sentences according to ``grammar.pest``.
So now we can build a database:

```rust
let kb = DBGenerator::gen_kb();
```
&nbsp;
&nbsp;

We can add data to it:

```rust
kb.tell("susan likes oranges.");
kb.tell("susan likes apples.");
kb.tell("john likes oranges.");
kb.tell("john hates apples.");
```
&nbsp;
&nbsp;

Finally we can query the system like:

```rust
kb.ask("john likes oranges.");  // -> true
kb.ask("john likes apples.");  // -> false
kb.ask("susan likes X1.");  // -> [{<X1>: oranges}, {<X1>: apples}]
kb.ask("X1 likes oranges. X1 likes apples.");  // -> [{<X1>: susan}]
kb.ask("susan likes X1. john likes X1.");  // -> [{<X1>: oranges}]
kb.ask("susan X1 apples. john X1 apples.");  // -> []
```

And that's it.

## Benchmarks

Work in progress.

Prelinary data shows that for very simple data (like the triple store above),
with 5.000.000 entries in the db, uindex performs about the same as (in memory, fully indexed) sqlite
for adding data, and outperforms it by about double for querying data.
It must also be said that at present, the uindex db weights about 5 times the sqlite db.

## TODO

Note that this is a work in progress. At the moment uindex does not even have persistence;
it only exists in memory. There is also room for improvement in the sizes of the dbs,
and queries would benefit by using some parallellism.

[0]:https://uindex.modus_ponens.net
[1]:https://en.wikipedia.org/wiki/Parsing_expression_grammar
[2]:https://pest.rs
[3]:https://www.rust-lang.org
[4]:https://pest.rs/book/grammars/syntax.html
