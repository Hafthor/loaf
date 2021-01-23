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
