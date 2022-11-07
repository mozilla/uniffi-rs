# [short title of solved problem and solution]

* Status: [proposed superseded by [gogo](0008-split-bindings-to-modules-automatically.md)]
* Deciders: []
* Date: [2022-07-11 when the decision was last updated]

## Context and Problem Statement

In his current state, uniffi only allows to import any generated code to a single file. This is hard to read and to maintain for a binding library user because in languages used to make bindings, the user has to import submodules.

Currently, I have to write:

from cryptatools_core.python3_bindings import CaesarNumberAlgorithm, Encoding, Alphabet

instead of:

from cryptatools_core.cryptography.encryption.monoalphabetic_ciphers import CaesarNumberAlgorithm
from cryptatools_core.utils.alphabets import Encoding, Alphabet

## Considered Options

* [option 1] : Force the user to rewrite all import in an __init__ for each python file. Mention it on the documentation. Not recommended.
* [option 2] : Fork Uniffi to write two new types of objects know as directory and file. EG:

directory house {
    file small_house {
        interface SmallHouse {
            constructor();
            [Name=build_house]
            constructor();
            string get_house_street();
        };
    }
}

* [option 3] : Fork `Uniffi` in order to let the user set an uniffi.toml in each module in order to define the modules in bindings. We could rely and modify this function: https://github.com/mozilla/uniffi-rs/blob/main/uniffi_bindgen/src/lib.rs#L188.

* [option 4] : Fork `Uniffi` in order to ask multiple `.udl` files for each module in the project.

In order to achieve this goal we need to first: https://github.com/mozilla/uniffi-rs/tree/main/uniffi_bindgen/src/interface

Then, edit each languages in https://github.com/mozilla/uniffi-rs/blob/main/uniffi_bindgen/src/bindings ans then 

 Recommended.

## Decision Outcome

Chosen option: "[option 1]", because [justification. e.g., only option, which meets k.o. criterion decision driver | which resolves force force | … | comes out best (see below)].

### Positive Consequences <!-- optional -->

* Reduce work time of the users.

### Negative Consequences <!-- optional -->

* Require time to code Uniffi.

## Pros and Cons of the Options <!-- optional -->

### [option 1]

[description] <!-- optional -->

* Good, because [It does not need more code in the uniffi codebase]
* Bad, because [The user will require useless time to code.]
* … <!-- numbers of pros and cons can vary -->

### [option 2]

[description] <!-- optional -->

* Good, because [The user will not require more time to code.]
* Bad, because [It will require more time for the developpers to code]
* Bad, because [We might want to do a refactoring on uniffi before to allow developper to add new languages more easily in Uniffi.]
<!-- numbers of pros and cons can vary -->

### [option 3]

[description] <!-- optional -->

* Good, because [there will be no need to extend the udl file.]
* Bad, because [It will only focus on rust modules. Not submodules.]
* Good, because there is already something in the uniffi.toml file. See: https://github.com/mozilla/uniffi-rs/blob/e260459fe947ce9058438e8bc3b9791345f2331f/docs/manual/src/swift/configuration.md

### [option 4]

[udl-files-everywhere] <!-- optional -->

* Good, because it seems very similar to the swig method and is probably the best one. See: https://www.swig.org/Doc1.1/HTML/Library.html#n7
* Good, because [Each folder will be listed.]
* Good, because if placed in each user project folder, the `.udl` files will correspond to each data structure in respective folder. Sounds more maintainable.
* Bad, because [May be redundant to have tons of `.udl` files in the user project.]
* Bad, because [May be hard to implement.]

<!-- numbers of pros and cons can vary -->


## Links <!-- optional -->

* [ADR-0008] [0008-split-bindings-to-modules-automatically.md] <!-- example: Refined by [ADR-0005](0005-example.md) -->
* … <!-- numbers of links can vary -->