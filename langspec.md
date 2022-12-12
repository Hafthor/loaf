# Language Specification

## Encoding
loaf excepts source files to be in UTF-8 encoding.

## Whitespace
Whitespace separate tokens in the source file but are ignored beyond that. A whitespace can be any of the following:
space, carriage return, line feed, form feed, tab, vertical tab or null and all are treated equally.

## Comments
Single line comments can be made with a hash symbol, such as

    statement   # this is a comment

Comments can also span multiple lines using multiple hash symbols to open/close a comment block. Comments are ignored and converted to whitespace. You cannot use comments to concatenate tokens like this:
   
    white## comment ##space: " \r\n\f\v\t\0"

## Identifiers
Identifiers can be any length string starting with:
 - a letter
 - an underscore
 - a noncombining alphanumeric Unicode character in the Basic Multilingual Plane (BMP)
 - a character outside the BMP that isn’t in a Private Use Area (PUA)
    
and can include digits and combining Unicode characters after the first character.

You can also use any sequence of UTF-8 characters, provided you enclose them in backticks (`). Backticks themselves can be part of an identifier if doubled, such as:

    `this is an identifier with a `` backtick character`

## Reserved Keywords
The following are reserved keywords and cannot be used as identifiers unless enclosed in backticks (`).

 - null
 - true
 - false

## Symbol Characters
The following symbol characters are reserved for use in loaf:

    !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~

the set includes all non-control non-digit non-letter printable characters in the ASCII character set.

 - ! is used for boolean NOT and as a part of != operator
 - " is used as a string delimiter
 - `#` is used as a comment character
 - $ is used to denote template strings
 - % is not presently used
 - & is used as a boolean AND operator
 - ' is used as a string delimiter
 - () are used to wrap expressions for operator precedence and for tuple wrapping
 - `*` is used for multiplication and intersectional operations
 - `+` is used for addition and concatenation operations
 - , is used to separate elements of an object, array or tuple
 - `-` is used for subtraction and removal operations
 - . is used to dereference objects
 - / is used to delimit regular expressions and as part of the // operator to denote HTTP operations
 - : is used as the declaration/assignment operator and as part of the ?: ternary expression
 - ; is not presently used
 - <> are used for quantitative comparison operations, including <= and >=
 - = is used as the equality comparison operator and as part of <= and >=
 - ? is used as the ternary operator and as part of the null dereferencing operator ?.
 - @ is used to denote literal strings
 - [] are used to wrap array expressions and for object/array/tuple dereferencing
 - \ is presently only used as an escape within non-literal strings and within regular expressions
 - ^ is presently not used
 - _ is used as a valid character for identifiers
 - ` is used to wrap identifiers that use otherwise invalid identifier characters
 - {} are used to wrap object expressions
 - ~ is presently not used

## Types
Below are the primitive types in loaf, which are essentially the same and derived from JSON:
 - number (not int or float, but decimal number with arbitrary size and precision)
 - string (not UTF-16, but essentially a byte array that can be processed as a stream of code points, runes or as graphemes.)
 - bool
 - array (a sparse array like object, like JavaScript's array type)
 - object (an object, somewhat like JavaScript's object type, but supports complex keys, not just strings)

along with those are some special types, like regular expressions, http request/response objects and functions.

Note that despite having an object type and supporting objects and arrays holding functions, loaf is NOT an object-oriented language.

## Literals
Literals include all of the 'specification' of JSON and includes objects, arrays, strings, numbers and the symbols true, false, and null.

### String Literals
String literals also include:
 - \x00 style byte codes
 - \U00000 5 hex digit style character codes
 - \U{0} 1 to 8 decimal digit style character codes, in addition to the
 - \u0000 style UTF-16 character codes, which should be avoided in loaf source as loaf is designed for UTF-8.

String literals are enclosed with either single (') or double quotes ("). There is no difference between them other than you have to escape the quote itself if used within the string.

You can prefix a string literal with an at sign (@) to disable backslash escapement.

    s: @"\backslash\"

You can prefix a string literal with a dollar sign ($) to allow for template substitutions.

    s: $"value of i is {i}"

### Number Literals
Number literals follow JSON and can include any number of digits before and after the decimal point. The decimal exponent, however, is limited to the size of a 64-bit signed integer in size.

### Regular Expression Literals
Regular expression literals are enclosed in slashes (/).

    isValidSecond: /[0-5][0-9]/.test(second)

### Array Literals
Array literals are enclosed by square brackets [] and items are delimited by commas (,). Extra trailing commas are ignored.

### Object Literals
Object literals are enclosed by curly braces {} and key/pairs are delimited by commas (,) and using colon (:) to separate key from value. Unlike JSON, object literal keys in loaf can be complex objects and not just strings. Extra trailing commas are ignored.

## Operators
Below are definitions for operators as defined by their source (left-value) operand. Generally, operators are plus (+) for adding two values, hyphen (-) for subtracting or removing, and asterisk (*) for multiplying or intersection.

### Number Operators
Below are the arithmetic operators that work on numbers:
 - plus (+) for addition
 - hyphen (-) for subtraction
 - asterisk (*) for multiplication
 
note that slash (/) is NOT used for division. This is done using the //math/divmod function.
note that neither caret (^) nor double asterisk (**) is used for exponentiation. This is done using the //math/pow function.

### String Operators
Below are the operators that work on strings:
 - plus (+) for concatenation
 - hyphen (-) for removal
 - asterisk (*) for intersection

### Array Operators
 - plus (+) for concatenation (array1 + array2 to append arrays, or array1 + non-array to add one more item to array1).
 - minus (-) to remove item (non-array) or items (array) from source array.
 - asterisk (*) for intersection of two arrays.

### Object Operators
 - plus (+) to merge two objects
 - minus (-) to remove key (non-array) or keys (array) from source object.
 - asterisk (*) for intersection of object and wanted array of keys.