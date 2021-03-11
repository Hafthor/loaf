loaf language - a json template based functional language for building restful json SoA orchestrations
  - forward assignments with automatic deferral resolution including serial/parallel async operations
  - native number type similiar to Java's BigDecimal
  - native string type is utf-8 with unicode spec iterators/streams (chars, graphemes)
  - bidirectional/collapseable enc/decoders
  - request/response unshared memory pools to lower copy/alloc/gc costs - only shared/long-lived memory is cache
  - functions are exportable as json restful endpoints
  - single runtime with common cache engine, live code change deployments
  - language level support for http symantics including caching, logging, metrics, injection, redirection
  - visualization tools to diagram dataflow, navigate architecture or watch/debug traffic
  - declarative same-source test specifications
  - automatic object property discovery / checking
  - support for streaming json responses as data becomes available

# Forward Assignments
In most languages, it is expected that you should be able to call a function/method that appears later in the file. This is a forward reference. Few languages support this for assignments, but loaf does. It can allow this because they are immutable assignments, not variable.

    ab: a+b
    a: 1
    b: 2
    
# Deferred Resolution
This is like Promises in JavaScript, but you can reference the Promise in an equation, for example, and that will seemlessly create another promise.

    ab: a+b
    a: service_a/get-a-value?x={x}
    b: service_b/get-b-value?x={x}
    x: service_x/get-x-value

Note that the ab is the same as before, however now it is a deferred value that waits on the deferred values of a and b which are deferred based on getting the value for x. loaf automatically makes the service calls when it can. In this example, the call to service_x is made, then the parallel calls to service_a and service_b are made. Then the value of ab is resolved.

# Numbers
The only number type in loaf is a magical unbounded number type that is similiar to Java's BigDecimal.

# Strings
The only string type in loaf is a essentially an array of bytes that is interpretted as UTF-8 text.

# Encoders/Decoders
There is special support for encoders/decoders that allows you to chain them, but also it supports collapsing them. For example, if you read from a query string parameter, and output that to a JSON string, that is done with a stack of a query string decoder with a JSON string encoder. If you, instead read from a JSON string and output to a different JSON string, the stack collapses and the transformation becomes a noop. These encoders/decoders are used to avoid creation of heap objects.

# Per Request Memory Pool
Rather than a single memory pool shared by the application, each external request creates a private pool. Once the response is sent, the private pool is discarded. There is no persistent memory, rather, loaf applications are expected to use the built-in cache. By having per-request memory pooling, the demands of garbage collection is greatly reduced. loaf also avoids creating heap objects. It is generally cheaper/faster to re-read and parse the source many times over, rather than making a parsed copy. loaf will initially use a naive approach where some things may be reprocessed many times. Future versions are expected to support internally caching these. Having per-request memory pooling also means that even when GC occurs, this does not stop the world, only the thread actively using pool.

# Exporting / Importing
By default, a service is private and only used on the same server and in the common loaf runtime. By exporting a service, you can allow it to be merely called over http by a foriegn service, or, if suitable, you can allow it to be consumed as code to be run on the foriegn server's loaf runtime.

When importing a service, this can be either a reference, which if supported will bring in discovery information for development purposes, or it can attempt to pull the code in to be run locally.

# Common loaf Runtime
You normally only have one copy of the loaf runtime operating. This is to allow cross-app calls with using http and allows for avoidance of forced early deferral resolution. The Common loaf Runtime also hosts a single common cache pool, which allows all the memory to be safely used. Cache is the only memory store in loaf that survives the request, and can be used as application state storage with infinite lifetime. Having a common runtime also allows portions of your loaf stack to be upgraded while running.

# HTTP Symantics
Because loaf is built for making an consuming web services, it is an http native. As such, it natively observes cache symantics using the common runtime cache pool. loaf also allow injection, logging, redirection and monitoring at any boundary that could be exported as http. This allows for data tracing, data flow diagraming, diagnosis and easy metrics reporting and alarming.

# Testing
Rather than testing being an afterthought, testing is built in. Tests are declared in the same source as the code they test and tests are just declarations of givens and expectations. Code-free tests.

# Automatic Property Discovery
Rather than strong-typing, loaf uses automatic typing to discover types in your code and code you import, both consuming or referencing imports. loaf warns you when you appear to reference a property that doesn't exist or has a different type. This feature is driven by tests which provide the prototypal responses you can expect as a caller.

# Support for Streaming JSON
As a response object is being put together, if the caller supports it, a streaming JSON result may be returned.

/: {
    ab: a+b
    a: service_a/get-a-value?x={x}
    b: service_b/get-b-value?x={x}
    x: service_x/get-x-value
}

let's suppose that service a is slower than b, what would be returned is:

    GET /
    ...wait
    200 OK
    {"x":123}\n ...wait
    {"b":222}\n ...wait
    {"a":111, "ab":333}

