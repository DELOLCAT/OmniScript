# Prelude

This document is about **platform files**, a feature that allows seamless and safe FFI, planned for the Rust rewrite of OmniScript (now called koniscript).

This document is not very extensive, and will be revised

This is the 3rd revision, and 1st rewrite of this document. Take a look at the previous rewrite for more info about topics not covered in here

## Planned Syntax

in `my_plt.knp`:

```koniscript platform
import root::other_plt::some_plt

@id 'io.github.koniscript.official.plt_files'

func foo(bar: str, optional foo: int) -> str # Functions are defined via regular declarations, with all features (like optional parameters, which convert to Optional<T>)

requirement foo {
    id: 'io.github.koniscript.official.custom_reqs
    type: standalone # Or `mutually_exclusive(req1, req2)`, with those reqs needing to be in scope
}

func bar() requires some_plt
```

Imported via

```koniscript
import my_plt

my_plt::foo('testing testing', 123)
```

## Native modules with fallbacks

Lets say that you have a module that can be native and also made in koniscript, like JSON.

For that module, you would first make a spec of functions, return types, failure modes, and more, and a platform file ID.

And a frontend module that works like this:

```koniscript
export func loads(input: str) -> JSONResponse # enum with Array(array) or Object(dict) {
    @platform native::json as nj {
        return nj::loads(str)
    } else @require <snip> { # (<snip> is NOT syntax, just a placeholder for docs)
        return kmodules::json::loads(str)
    }
}
```