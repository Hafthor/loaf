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

Note: regarding the examples below, the syntax hasn't been entirely nailed down, so don't get too attached to it.

# Forward Assignments
In most languages, it is expected that you should be able to call a function/method that appears later in the file. This is a forward reference. Few languages support this for assignments, but loaf does. It can allow this because they are immutable assignments, not variable.

    total: subtotal + tax
    tax: subtotal * taxrate
    taxrate: 0.10
    subtotal: price * qty
    price: 123.00
    qty: 3
    
# Deferred Resolution
This is like Promises in JavaScript or Futures in Java, but you can reference the Promise in an equation, for example, and that will seemlessly create another promise. Essentially all assignments are deferred, but some become instantly resolved when they have no deferred values referenced in their expression.

    total: subtotal + tax
    tax: subtotal * taxrate
    taxrate: //tax_service/get?zip={cust.addr.zip}
    subtotal: price * {qty}
    price: //price_service/get?cust={cust.id}
    cust: //customer_service/get-customer?user={user}&password={pwd}

Note that the total is the same as before, however now it is a deferred value that waits on the deferred values of taxrate and price which are deferred based on getting the value for cust. loaf automatically makes the service calls when it can. In this example, the call to customer_service is made, then the parallel calls to tax_service and price_service are made. Then the value of total is resolved.

# Numbers
The only number type in loaf is a magical unbounded number type that is similiar to Java's BigDecimal.

    nickel: 0.05
    dime: nickel + nickel
    total: nickel + dime

    {"nickel": 0.05, "dime": 0.10, "total": 0.15}
    
Note that dime is 0.10 vs 0.1. Also note that total is 0.15 rather than 0.15000000000000002.

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
      ab: a + b
      a: //service_a/get-a-value?x={x}
      b: //service_b/get-b-value?x={x}
      x: //service_x/get-x-value
    }

let's suppose that service a is slower than b, what would be returned is:

    GET /
    ...wait
    200 OK
    {"x":123}\n ...wait
    {"b":222}\n ...wait
    {"a":111, "ab":333}

# Origins of loaf
loaf came out of a desire to easily orchestrate web services. When I was an consultant building iOS apps, or web SPAs, I would have code that was making calls to the server, but I wanted a way to efficiently put together all the data in one response. One user action, one http call. I considered building some sort of DSL for this, but then I was exposed to XSLT which got me thinking of a more declarative approach. loaf began as something like https://jsonnet.org.

The per-request memory pooling came as I was building some applications that used a ridiculous amount of memory and how GC was pausing the world for many seconds. I found this silly given how there was almost no memory actually being shared between requests. I started thinking of how to introduce memory pooling into an existing language like Java, but this would have been a daunting task.

Many of the other features of loaf just came from my years of experience and my bug bears about development.

# On Syntax
Of primary importance is readability. Part of that is avoidance of non-value added noise. This includes the requirement of named types. The syntax tries to be obvious. The language should be powerful, but not at the expense of encouraging opaque 'design patterns' like dependency injection. Design patterns are often evidence of language weakness. loaf is designed to support the goal of the patterns without the developer having to do it.

If possible, loaf will avoid having any keywords. If possible, loaf will allow variables with spaces, although this is a stretch goal.
