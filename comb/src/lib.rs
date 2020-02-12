pub type ParseResult<'a, T> = Result<(&'a str, T), (&'a str, ParserError)>;

/// Recognizes a fixed string pattern.
///
/// If the input data matches the first argument, it will return a successful
/// value containing the argument itself.
///
/// Otherwise it returns `Err((_, ParserError::Tag))`
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = tag("function");
///
/// assert_eq!(parser("function hello"), Ok((" hello", "function")));
/// assert_eq!(parser("Something else"), Err(("Something else", ParserError::Tag)));
/// assert_eq!(parser(""), Err(("", ParserError::Tag)));
/// ```
pub fn tag(tag: &'static str) -> impl Fn(&str) -> ParseResult<&str> {
    move |i| match i.starts_with(tag) {
        true => Ok((&i[tag.len()..], tag)),
        false => Err((i, ParserError::Tag)),
    }
}

/// Converts a matched parser to a fixed value.
///
/// It doesn't produce errors by itself, but errors by the inner parser are
/// forwarded to the output.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = value(tag("Hello, world!"), "Hallo welt");
///
/// assert_eq!(parser("Hello, world!"), Ok(("", "Hallo welt")));
/// assert_eq!(parser("Bonjour le monde"), Err(("Bonjour le monde", ParserError::Tag)));
/// ```
pub fn value<'a, P, R, V>(p: P, v: V) -> impl Fn(&'a str) -> ParseResult<V>
where
    P: Fn(&'a str) -> ParseResult<R>,
    V: Copy,
{
    move |i| p(i).map(|(i, _)| (i, v))
}

/// Transforms the (successful) output of a matched parser.
///
/// Since parsers dwell mostly in strings, you need something to turn them into
/// other kinds of values. As with the `value` combinator, errors by the inner
/// parser are forwarded to the output.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = map(tag("1"), |s| s.parse::<i32>().unwrap());
///
/// assert_eq!(parser("1"), Ok(("", 1)));
/// assert_eq!(parser("2"), Err(("2", ParserError::Tag)));
/// ```
pub fn map<'a, P, F, A, B>(p: P, f: F) -> impl Fn(&'a str) -> ParseResult<B>
where
    P: Fn(&'a str) -> ParseResult<A>,
    F: Fn(A) -> B,
{
    move |i| p(i).map(|(i, r)| (i, f(r)))
}

/// Same as the map combinator, but errors in the lambda function used to
/// transform cause the parser to reject the token.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = map_res(take_while(|c| c.is_alphanumeric()), |s| s.parse::<i32>());
///
/// assert_eq!(parser("123"), Ok(("", 123)));
/// assert_eq!(parser("abc"), Err(("", ParserError::MapRes)));
/// ```
pub fn map_res<'a, P, F, A, B, E>(p: P, f: F) -> impl Fn(&'a str) -> ParseResult<B>
where
    P: Fn(&'a str) -> ParseResult<A>,
    F: Fn(A) -> Result<B, E>,
{
    move |i| p(i).and_then(|(i, r)| f(r).map(|r| (i, r)).or(Err((i, ParserError::MapRes))))
}

/// Makes the inner parser optional by swallowing errors and turning them into a
/// `None` value. Actual matched values are boxed by `Some`.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = opt(tag("1"));
///
/// assert_eq!(parser("1"), Ok(("", Some("1"))));
/// assert_eq!(parser("2"), Ok(("2", None)));
/// ```
pub fn opt<'a, P, R>(p: P) -> impl Fn(&'a str) -> ParseResult<Option<R>>
where
    P: Fn(&'a str) -> ParseResult<R>,
{
    move |i| p(i).and_then(|(i, r)| Ok((i, Some(r)))).or(Ok((i, None)))
}

/// Matches a pair of tokens. Both have to matched, otherwise the wole thing
/// fails. The error returned by the first unmatched parser is returned.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = pair(tag("hello "), tag("world"));
///
/// assert_eq!(parser("hello world"), Ok(("", ("hello ", "world"))));
/// assert_eq!(parser("oh noes"), Err(("oh noes", ParserError::Tag)));
/// ```
pub fn pair<'a, A, B, X, Y>(a: A, b: B) -> impl Fn(&'a str) -> ParseResult<(X, Y)>
where
    A: Fn(&'a str) -> ParseResult<X>,
    B: Fn(&'a str) -> ParseResult<Y>,
{
    move |i| a(i).and_then(|(i, r1)| b(i).map(|(i, r2)| (i, (r1, r2))))
}

/// What's better than a pair? You got it: a trio.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = trio(tag("ein "), tag("zwei "), tag("drei"));
///
/// assert_eq!(parser("ein zwei drei"), Ok(("", ("ein ", "zwei ", "drei"))));
/// assert_eq!(parser("one two three"), Err(("one two three", ParserError::Tag)));
/// ```
pub fn trio<'a, A, B, C, X, Y, Z>(a: A, b: B, c: C) -> impl Fn(&'a str) -> ParseResult<(X, Y, Z)>
where
    A: Fn(&'a str) -> ParseResult<X>,
    B: Fn(&'a str) -> ParseResult<Y>,
    C: Fn(&'a str) -> ParseResult<Z>,
{
    move |i| a(i).and_then(|(i, x)| b(i).and_then(|(i, y)| c(i).map(|(i, z)| (i, (x, y, z)))))
}

/// Just like the pair combinator, but it throws away the result of the parser
/// in the right and returns a single value.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = right(tag("not me "), tag("me"));
///
/// assert_eq!(parser("not me me"), Ok(("", "me")));
/// assert_eq!(parser("not me you"), Err(("you", ParserError::Tag)));
/// ```
pub fn right<'a, A, B, X, Y>(a: A, b: B) -> impl Fn(&'a str) -> ParseResult<Y>
where
    A: Fn(&'a str) -> ParseResult<X>,
    B: Fn(&'a str) -> ParseResult<Y>,
{
    move |i| a(i).and_then(|(i, _)| b(i).map(|(i, r2)| (i, r2)))
}

/// We already have a right combinator. Guess what's next? The left. Balanced,
/// as all things should be
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = left(tag("me"), tag("you"));
///
/// assert_eq!(parser("meyou"), Ok(("", "me")));
/// assert_eq!(parser("youme"), Err(("youme", ParserError::Tag)));
/// ```
pub fn left<'a, A, B, X, Y>(a: A, b: B) -> impl Fn(&'a str) -> ParseResult<X>
where
    A: Fn(&'a str) -> ParseResult<X>,
    B: Fn(&'a str) -> ParseResult<Y>,
{
    move |i| a(i).and_then(|(i, r1)| b(i).map(|(i, _)| (i, r1)))
}

/// Same as left and right, but now it rejects both tokens that bookend the one
/// in the middle.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = middle(tag("("), tag("secret"), tag(")"));
///
/// assert_eq!(parser("(secret)"), Ok(("", "secret")));
/// assert_eq!(parser("secret"), Err(("secret", ParserError::Tag)));
/// ```
pub fn middle<'a, A, B, C, X, Y, Z>(a: A, b: B, c: C) -> impl Fn(&'a str) -> ParseResult<Y>
where
    A: Fn(&'a str) -> ParseResult<X>,
    B: Fn(&'a str) -> ParseResult<Y>,
    C: Fn(&'a str) -> ParseResult<Z>,
{
    move |i| a(i).and_then(|(i, _)| b(i).and_then(|(i, r2)| c(i).map(|(i, _)| (i, r2))))
}

/// Takes the result of the outermost parsers and rejects the middle. Useful for
/// parsing separated pairs.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = outer(tag("a"), tag(","), tag("b"));
///
/// assert_eq!(parser("a,b"), Ok(("", ("a", "b"))));
/// assert_eq!(parser("a+b"), Err(("+b", ParserError::Tag)));
/// ```
pub fn outer<'a, A, B, C, X, Y, Z>(a: A, b: B, c: C) -> impl Fn(&'a str) -> ParseResult<(X, Z)>
where
    A: Fn(&'a str) -> ParseResult<X>,
    B: Fn(&'a str) -> ParseResult<Y>,
    C: Fn(&'a str) -> ParseResult<Z>,
{
    move |i| a(i).and_then(|(i, x)| b(i).and_then(|(i, _)| c(i).map(|(i, z)| (i, (x, z)))))
}

/// Tries to match either one of the parsers and returns the sucessful one.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = either(tag("a"), tag("b"));
///
/// assert_eq!(parser("a"), Ok(("", "a")));
/// assert_eq!(parser("b"), Ok(("", "b")));
/// assert_eq!(parser("c"), Err(("c", ParserError::Tag)));
/// ```
pub fn either<'a, A, B, R>(a: A, b: B) -> impl Fn(&'a str) -> ParseResult<R>
where
    A: Fn(&'a str) -> ParseResult<R>,
    B: Fn(&'a str) -> ParseResult<R>,
{
    move |i| a(i).or_else(|_| b(i))
}

/// Exactly like either, but in this case you can have as many choices as you
/// need (as long as they have the same type).
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = choice([ tag("1"), tag("2"), tag("3") ]);
///
/// assert_eq!(parser("1"), Ok(("", "1")));
/// assert_eq!(parser("2"), Ok(("", "2")));
/// assert_eq!(parser("3"), Ok(("", "3")));
/// assert_eq!(parser("4"), Err(("4", ParserError::Choice)));
/// ```
pub fn choice<'a, S, P, R>(ps: S) -> impl Fn(&'a str) -> ParseResult<R>
where
    S: AsRef<[P]>,
    P: Fn(&'a str) -> ParseResult<R>,
{
    move |i| {
        AsRef::as_ref(&ps)
            .iter()
            .find_map(|p| p(i).ok())
            .ok_or((i, ParserError::Choice))
    }
}

/// This one swallows characters as long as a condition is matched.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = take_while(|c| c.is_numeric());
///
/// assert_eq!(parser("123"), Ok(("", "123")));
/// assert_eq!(parser("456"), Ok(("", "456")));
/// assert_eq!(parser("abc"), Err(("abc", ParserError::TakeWhile)));
/// ```
pub fn take_while<'a, P>(p: P) -> impl Fn(&'a str) -> ParseResult<&str>
where
    P: Copy + Fn(char) -> bool,
{
    move |i| match i.find(|c| !p(c)) {
        Some(0) => Err((i, ParserError::TakeWhile)),
        Some(x) => Ok((&i[x..], &i[..x])),
        None if i.len() > 0 => Ok((&i[i.len()..], i)),
        None => Err((i, ParserError::TakeWhile)),
    }
}

/// Matches and joins all matched characters until the combination of characters
/// in the first argument is reached. Don't worry, it can be just one character
/// if you want.
///
/// This is very forgiving about errors. If the end of the string comes before
/// the breaking combination, it will return everything it found.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = take_until(" combo breaker");
///
/// assert_eq!(parser("123 combo breaker"), Ok((" combo breaker", "123")));
/// assert_eq!(parser("456"), Ok(("456", "")));
/// ```
pub fn take_until<'a>(p: &'a str) -> impl Fn(&'a str) -> ParseResult<&str> {
    move |i| i.find(p).map_or(Ok((i, "")), |x| Ok((&i[x..], &i[..x])))
}

/// A curious parser: it returns successful response if its inner parsers are
/// able to find something, but it doesn't eat the characters.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = peek(tag("The future"));
///
/// assert_eq!(parser("The future"), Ok(("The future", "The future")));
/// assert_eq!(parser("Not the future"), Err(("Not the future", ParserError::Tag)));
/// ```
pub fn peek<'a, P, R>(p: P) -> impl Fn(&'a str) -> ParseResult<R>
where
    P: Fn(&'a str) -> ParseResult<R>,
{
    move |i| p(i).map(|(_, o)| (i, o))
}

/// This one extract the raw characters that were matched by the inner parsers.
///
/// It is particularly useful when you want to meticulously parse some content,
/// but wants it's raw content instead.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = recognize(pair(tag("badger"), tag("badger")));
///
/// assert_eq!(parser("badgerbadger"), Ok(("", "badgerbadger")));
/// assert_eq!(parser("mushroom"), Err(("mushroom", ParserError::Tag)));
/// ```
pub fn recognize<'a, P, R>(p: P) -> impl Fn(&'a str) -> ParseResult<&'a str>
where
    P: Fn(&'a str) -> ParseResult<R>,
{
    move |i| p(i).map(|(i2, _)| (i2, &i[..(i2.as_ptr() as usize - i.as_ptr() as usize)]))
}

/// Ensures that the return of an inner parser matches the stated conditions.
///
/// Example
/// ```rust
/// use comb::*;
///
/// let parser = check(take_until("-"), |a| a.len() == 3);
///
/// assert_eq!(parser("yes-"), Ok(("-", "yes")));
/// assert_eq!(parser("no-"), Err(("no-", ParserError::Check)));
/// ```
pub fn check<'a, P, R, F>(p: P, f: F) -> impl Fn(&'a str) -> ParseResult<R>
where
    P: Fn(&'a str) -> ParseResult<R>,
    F: Fn(&R) -> bool,
{
    move |i| match p(i) {
        Ok((i, r)) if f(&r) => Ok((i, r)),
        _ => Err((i, ParserError::Check)),
    }
}

/// Probably the most useful combinator of all. It matches multiple instances
/// the same sequence. Very useful for everything, from numbers and strings, up
/// to complex sequences.
///
/// If it doesn't find any instance of, the sequence it just returns an empty
/// array.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = many(tag("badger"));
///
/// assert_eq!(parser("badgerbadgerbadger"), Ok(("", vec!["badger", "badger", "badger"])));
/// assert_eq!(parser("not badger"), Ok(("not badger", vec![])));
/// ```
pub fn many<'a, P, R>(p: P) -> impl Fn(&'a str) -> ParseResult<Vec<R>>
where
    P: Fn(&'a str) -> ParseResult<R>,
{
    move |mut i| {
        let mut r = Vec::new();
        while let Ok((next_input, next_item)) = p(i) {
            i = next_input;
            r.push(next_item);
        }
        Ok((i, r))
    }
}

/// Matches a chain of repetitions linked (got it?) by a certain token or
/// combination of tokens.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = chain(tag(","), tag("1"));
///
/// assert_eq!(parser("1,1,1"), Ok(("", vec!["1", "1", "1"])));
/// assert_eq!(parser("1,1,1,"), Ok(("", vec!["1", "1", "1"])));
/// assert_eq!(parser("1"), Ok(("", vec!["1"])));
/// assert_eq!(parser("2"), Ok(("2", vec![])));
/// ```
pub fn chain<'a, S, P, R1, R2>(sep: S, p: P) -> impl Fn(&'a str) -> ParseResult<Vec<R2>>
where
    S: Fn(&'a str) -> ParseResult<R1>,
    P: Fn(&'a str) -> ParseResult<R2>,
    R1: Clone,
    R2: Clone,
{
    let join = |(a, b)| [vec![a], b].concat();
    move |i| map(pair(&p, left(many(right(&sep, &p)), opt(&sep))), join)(i).or(Ok((i, vec![])))
}

/// Similar to the chain operator, but it stores the result of the infix
/// operators.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = infix(double, tag("+"));
///
/// assert_eq!(parser("1+1"), Ok(("", (1.0, vec![("+", 1.0)]))));
/// assert_eq!(parser("1+1+2"), Ok(("", (1.0, vec![("+", 1.0), ("+", 2.0)]))));
/// ```
pub fn infix<'a, P, O, R, S>(p: P, o: O) -> impl Fn(&'a str) -> ParseResult<(R, Vec<(S, R)>)>
where
    P: Fn(&'a str) -> ParseResult<R>,
    O: Fn(&'a str) -> ParseResult<S>,
{
    move |i| pair(&p, many(pair(w(&o), &p)))(i)
}

/// Similar to the infix operator, but allows multiple prefixes that can later
/// be chained together.
///
/// # Example
/// ```rust
/// use comb::*;
///
/// let parser = prefix(tag("!"), double);
///
/// assert_eq!(parser("!!!1"), Ok(("", (vec!["!", "!", "!"], 1.0))));
/// ```
pub fn prefix<'a, P, Q, X, Y>(p: P, q: Q) -> impl Fn(&'a str) -> ParseResult<(Vec<X>, Y)>
where
    P: Fn(&'a str) -> ParseResult<X>,
    Q: Fn(&'a str) -> ParseResult<Y>,
{
    move |i| pair(many(w(&p)), &q)(i)
}

/// Wraps the result of the inner parser in a plain-old Rust Box.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = boxed(tag("thing"));
///
/// assert_eq!(parser("thing"), Ok(("", Box::new("thing"))));
/// assert_eq!(parser("not thing"), Err(("not thing", ParserError::Tag)));
/// ```
pub fn boxed<'a, P, R>(i: P) -> impl Fn(&'a str) -> ParseResult<Box<R>>
where
    P: Fn(&'a str) -> ParseResult<R>,
{
    map(i, Box::new)
}

/// Parses a 64-bit floating point number.
///
/// # Example
/// ```
/// use comb::*;
///
/// assert_eq!(double("1.0"), Ok(("", 1.0)));
/// assert_eq!(double("2"), Ok(("", 2.0)));
/// assert_eq!(double("2e2"), Ok(("", 200.0)));
/// assert_eq!(double("2e-2"), Ok(("", 0.02)));
/// assert_eq!(double("-2"), Ok(("", -2.0)));
/// assert_eq!(double("+2"), Ok(("", 2.0)));
/// assert_eq!(double("+.2"), Ok(("", 0.2)));
/// assert_eq!(double("-.2"), Ok(("", -0.2)));
/// ```
pub fn double<'a>(i: &'a str) -> ParseResult<f64> {
    let digit = |i| take_while(|c| c.is_numeric())(i);
    let sign = |i| opt(either(tag("+"), tag("-")))(i);
    let num = value(pair(digit, opt(pair(tag("."), opt(digit)))), 0);
    let frac = value(pair(tag("."), digit), 0);
    let exp = opt(trio(either(tag("e"), tag("E")), sign, digit));
    map_res(recognize(trio(sign, either(num, frac), exp)), |s| s.parse())(i)
}

/// This is not a combinator, but rather a parser itself that detects if
/// we have reached the end of the input.
///
/// # Example
/// ```
/// use comb::*;
///
/// assert_eq!(eoi(""), Ok(("", "")));
/// assert_eq!(eoi("not the end"), Err(("not the end", ParserError::Eof)));
/// ```
pub fn eoi(i: &str) -> ParseResult<&str> {
    match i.is_empty() {
        true => Ok((i, "")),
        false => Err((i, ParserError::Eof)),
    }
}

/// A parser that only cares about whitespace! Spaces, tabs, line breaks.
/// All the invisible stuff will be eaten by this monster.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser1 = right(whitespace, tag("hello"));
/// let parser2 = left(tag("hello"), whitespace);
///
/// assert_eq!(parser1("    hello"), Ok(("", "hello")));
/// assert_eq!(parser2("hello    "), Ok(("", "hello")));
/// assert_eq!(whitespace(" \t\r\n "), Ok(("", " \t\r\n ")));
/// ```
pub fn whitespace<'a>(i: &str) -> ParseResult<&str> {
    match i.find(|c: char| !c.is_whitespace()) {
        Some(x) => Ok((&i[x..], &i[..x])),
        _ => Ok(("", i)),
    }
}

/// A convenient one-letter helper to trim whitespace from the left of your
/// tokens.
///
/// # Example
/// ```
/// use comb::*;
///
/// let parser = w(tag("tag"));
///
/// assert_eq!(parser("    tag"), Ok(("", "tag")));
/// ```
pub fn w<'a, P, R>(p: P) -> impl Fn(&'a str) -> ParseResult<R>
where
    P: Fn(&'a str) -> ParseResult<R>,
{
    right(whitespace, p)
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    Check,
    Choice,
    Eof,
    Tag,
    TakeWhile,
    MapRes,
}
