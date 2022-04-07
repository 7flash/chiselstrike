# Evolving types with a git-like workflow

## Goals:

* Extreme velocity when making type changes.
* Good production performance when, at a specified time, the user reaches _production mode_.
* git-like workflow for dealing with data inspection and transformations

Gist:

Much like we already do for fields in the types, each Typescript type matches internally to
a ChiselStrike type, with a unique ID. So for example, imagine the following type (or class,
part of the goal at this moment is find out which works best)

```
class Person {
    name: string = "";
    country: string = "";
}
```

This would be mapped out internally to a type called `ChiselType_Person_045ca25301f`. In the general
case, `chisel apply` will ingest the type, and see if the type matches exactly the current definition
for that type (or if the type is being created). Exceptions are labels, which can always be added or
removed.

`chisel apply` would output (note that the name is sugared for the user):

```
Type defined: Person@045ca25301f
```

When the user calls `chisel apply` again, with the following type (noting that one of the types is no longer default,
and became an integer):

```
class Person {
	name: string = "";
	country_code: int;

	constructor(code: int) {
		this.country_code = code;
        }
}
```

two things happen:

1. ChiselStrike generates a new internal type for that, say `ChiselType_Person_a7188a89e8`. 
2. ChiselStrike errors out `chisel apply`, with the following error message:

```
Could not apply current class `Person`. A version (`Person@045ca25301f` already exists), with the following definition:


 class Person {
     name: string = "";
     country: string = "";
 }

Please provide a function with the following signature:

function convertPerson(old: Person_id_045ca25301f): Person;

```

It seems to me that the IDs should be just generated by chiseld. It doesn't make sense to tie it to
user's git workflows because those can be rebased, history rewritten, not to mention there can be many commits
in which the type doesn't change.

There is precedence for create-on-apply: terraform creates resource IDs when resources are created and those are
output to the user.

One possible implementation of this function is:

```
function convertPerson(old: Person_id_045ca25301f): Person {
  let country = some_aux_function(old.country);
  let p = Person(country);
  p.name = old.name;
  return p;
}
```

This function is then added to `lenses/convert.ts`, and the next invocation of `chisel apply` will succeed.

Note that the user only deal with a single transformation at a time. Chiseld can persist the transformation history,
since that is tied to the state of the database (including whether or not it has data for the types). If the type
changes again, the user will write:

```
function convertPerson(old: Person_id_a7188a89e8): Person;
```

## Applying the lenses

* For the development workflow, lenses will be called explicitly for each element, much like what we do for
  policies. The database will still have the old values, but before exposing them to the endpoints they transformed.
* For the production workflow, `chisel type commit` will apply the transformations into the database (potentially with a
  long migration)

## Other New commands:

*  `chisel type log` will print a git-log version of how did the types evolved, as well as what is the current type

## Open questions

* Is there a story for branching and merging? 