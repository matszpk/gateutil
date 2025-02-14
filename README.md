## GateUtil

The library of basic functions that operates on circuits provided by gatesim.
This library provide set of functions that doing some operations on circuits.
Routines provided to make following operations:
* translate circuits inputs and circuit outputs.
* fill outputs defined in output map.
* generate circuits that have assigned values to original inputs.
* generate optimized circuits that have assigned values to original inputs.
* generate optimized and deduplicated circuits that have assigned values to original inputs.
* optimize clause circuit.
* deduplicate circuit or clause circuit.
* optimize and deduplicate clause circuit.
* generate circuit with negated inputs.
* generate join of two circuits sequentially.
* generate join of many circuits sequentially.
* calculate minimal and maximal depth of circuit.
* calculate minimal and maximal depth and depths for any gate of circuit.
* generate pipelined circuit.
* join input or output maps.

Some WARNINGS about using some routines. A optimization and deduplication routines
are not completely tested and they can get wrong results in some specific input.
They shouldn't be used in deployed software.
