# The Why of loaf
This document outlines the motivations of the features of loaf.

## Primary Use Case
The primary use case of loaf is to create web service orchestrations; something that is a RESTful JSON web service that can consume and process other RESTful JSON web services to create a RESTful JSON response.

## Declarative
A chief design goal is to make the language declarative, in the same way that SQL is declarative. You describe what you want done, not how you want it done. While highly successful in the database world, this type of software development has not really broken into common use.

Part of this declaration is processing non-immediate results, such as from web service calls. The developer should be able to simply state that the result is received from a web service call, and not have to worry about what needs to be called when and which ones can be done concurrently and which ones must be done serially.

This declarative approach is also extended to unit testing. Rather than writing imperative code that does setup, execute, assert type logic, tests are written as givens and expectation. This lowers the concern about logic errors in tests allowing code errors to escape detection.

While the design is declarative, it is also functional. Part of this is that it heavily favors immutability. By doing this, loaf can allow for reordering of declaration such that the developer is free to arrange code to show the main purpose of a method, with the details following in a way that would have ordinarily required breaking out to private helper type methods.

## One Number Type
Give a developer the choice of number types to use for a particular use case, they will almost certainly pick the wrong type. Mostly, this is a result of picking an integer number type that is eventually proved to be insufficient, or they choose floating point numbers which are wholly unsuited for doing math on decimal numbers. Computers are supposed to be good at numbers, so why are we still fighting to write simple business applications with number types that are unsuited to the task. loaf attempts to fix this with a bit of a brute force approach. All numbers are decimal floating point numbers with essentially unbounded scale and precision. The compiler and runtime could implement optimizations that use more machine primitive number types to realize the operations requested, but that should not be the concern of the business applications developer.

## Strings
UTF-8 won. Most web services accept and return UTF-8. By making loaf strings UTF-8, we avoid the essentially useless transcoding to UTF-16.

## Multiple Heaps
Given loaf's primary use case, being an orchestrator of web services, it is important to notice that, if it were written in any other high-level language, the request would come in, processing would be done, and the response would be written out and then essentially all of the objects created on the heap for and during that request are discarded. By making a separate heap for each request, we isolate the places where garbage collection has to occur and we can avoid garbage collection once the response is returned.

## Unified Cache / Runtime Daemon
Caching is often desired at various levels, but often caching is done in isolation and often without knowing the sizes of objects in cache. As a result, memory is often greatly under-utilized so as to avoid out of memory errors. A unified single cache operating in the runtime daemon can take advantage of all of the remaining system memory and do so in a manner that is safe to the other memory demands of the runtime. It can even prune cache in response to operating system signals of memory pressure from other non-loaf applications running.

## Same File Testing
Tests should be in the same file as the code being tested. Tests are more likely to be maintained this way. Given loaf's declarative testing, this also serves as documentation and can be used by development tools to automatically derive type inference information. Potentially, these tests could even be used to create graphs of systems built in loaf, showing how parts of the code are interlinked in a way potentially more meaningful than simple call graphs or code layer diagrams.

## Why the name 'loaf'?
loaf (always spelled all lowercase) isn't an acronym, but if it was it might stand for "lazy, obvious AF", for its lazy evaluation and, hopefully, obvious programs it allows us to write. Mostly it is called loaf because it was a name not taken or used by any other major technology and it should allow us to create a cute graphic brand image.

## Why is port 4271 the default?
4271 is 10af in hex, which kinda looks like the word loaf.
