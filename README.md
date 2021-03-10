loaf language - a json template based functional language for building restful json SoA orchestrations
  - forward assignments with automatic deferral resolution including serial/parallel async operations
  - native number type similiar to Java's BigDecimal
  - native string type is utf-8 with unicode spec iterators/streams (chars, graphemes)
  - bidirectional/collapseable enc/decoders
  - functions are exportable as json restful endpoints
  - language level support for http symantics including caching, logging, metrics, injection, redirection
  - single runtime with common cache engine
  - visualization tools to diagram dataflow, navigate architecture or watch/debug traffic
  - declarative same-source test specifications
  - automatic object property discovery / checking
  - live code change deployments
  - request/response unshared memory pools to lower copy/alloc/gc costs - only shared/long-lived memory is cache
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
There is special support for encoders/decoders that allows you to chain them, but also it supports collapsing them. For example, if you read from a query string parameter, and output that to a JSON string, that is done with a stack of a query string decoder with a JSON string encoder. If you, instead read from a JSON string and output to a different JSON string, the stack collapses and the transformation becomes a noop.

# Exporting / Importing
By default, a service is private and only used on the same server and in the common loaf runtime. By exporting a service, you can allow it to be merely called over http by a foriegn service, or, if suitable, you can allow it to be consumed as code to be run on the foriegn server's loaf runtime.

When importing a service, this can be either a reference, which if supported will bring in discovery information for development purposes, or it can attempt to pull the code in to be run locally.

# Common loaf Runtime
You normally only have one copy of the loaf runtime operating.
