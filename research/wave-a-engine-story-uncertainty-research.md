---
document_type: research
producer: research-agent
date: 2026-07-24
topic: Wave-A engine story pre-TDD uncertainty resolution (RFC 6265 cookie-name charset vs HeaderValue rejection; edition-2024 dyn-compatibility of defaulted boxed-future trait methods; chrono 0.4.44 FromStr leniency; serde default-on-Option; thiserror 1.0.69 -> 2.x Display deltas)
status: complete
research_type: general
research_pass: dclaude:remove-uncertainty (pre-TDD gate)
mcp_tool_calls: 11
training_data_reliance: low
verification_basis: primary — vendored crate sources under ~/.cargo/registry read directly at the exact locked versions; secondary — RFC text, Rust Reference, upstream release notes
questions_resolved: 5 of 5 (RQ-1 HIGH, RQ-2 MEDIUM, RQ-3 MEDIUM, RQ-4 LOW, RQ-5 LOW)
inconclusive_items: 2 (chrono introduction-version attribution for `from_timestamp_secs` and for the relaxed-RFC3339 `FromStr` switch — behaviour in 0.4.44 is verified, the introducing release is not)
---

# Wave-A Engine Story — Pre-TDD Uncertainty Resolution (2026-07-24)

This artifact resolves the five external-research questions raised by the pre-TDD uncertainty scan.
Each section carries the **QUESTION**, a direct **ANSWER**, **EVIDENCE** with citations, a
**VERSION-SPECIFICITY** note, and an **IMPLICATION FOR IMPLEMENTATION**.

> **Verification bias.** For every question that could be settled by reading the crate itself, I read
> the vendored source at the exact locked version under
> `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` rather than relying on prose docs or
> model knowledge. Where a claim rests on a source read, the section says so and gives the file and
> line. Two sub-items (chrono introduction versions) could not be settled from available sources and
> are flagged INCONCLUSIVE.

## Lockfile reconciliation (read before acting on version claims)

The task statement supplied a version set. I verified each against
`/Users/jmagady/Dev/prism/Cargo.lock`. Three corrections and one addition matter:

| Crate | Task statement | `Cargo.lock` actual | Note |
|---|---|---|---|
| serde | 1.0.228 | **1.0.228** | confirmed (`Cargo.lock:6128`) |
| chrono | 0.4.44 | **0.4.44** | confirmed (`Cargo.lock:1002`) |
| proptest | 1.11.0 | **1.11.0** | confirmed (`Cargo.lock:5304`) |
| thiserror | 1.0.69 | **1.0.69 AND 2.0.18** | **both majors are in the graph** (`Cargo.lock:6505`, `:6514`). The "one major version behind" framing in RQ-5 is only true of the *first-party* dependency edge; 2.0.18 is already vendored transitively. |
| toml | 0.8.23 | **0.8.23 AND 0.9.12+spec-1.1.0** | both present (`Cargo.lock:6694`, `:6706`) |
| `http` | (not given) | **1.4.0** | load-bearing for RQ-1 |
| `reqwest` | (not given) | **0.12.28** | load-bearing for RQ-1 |
| `toml_edit` | (not given) | **0.22.27** | load-bearing for RQ-4 serialize side; `toml` 0.8.23 delegates all serialization to it (`toml-0.8.23/Cargo.toml:153-157`, `display = ["dep:toml_edit", ...]`) |

Because two thiserror majors and two toml majors coexist, any story that pins "the" version must
name the crate edge, not just the crate.

---

## RQ-1 (HIGH) — RFC 6265 cookie-name charset vs `HeaderValue` rejection

### QUESTION

A validation rule accepts a cookie name as any non-empty string containing no colon. The name is
interpolated as `format!("{name}={token}")` and passed to `reqwest`'s `.header("Cookie", ...)`.
What charset does RFC 6265 §4.1.1 actually permit? Does `HeaderValue` construction reject or
sanitize control characters / spaces / non-token bytes? Where does the failure surface?

### ANSWER

Three-part answer, and the middle part is the dangerous one.

**(a) RFC 6265 `cookie-name` permits exactly 77 characters.** `cookie-name = token`, and `token`
is `1*tchar` where `tchar` is the 15 punctuation marks `! # $ % & ' * + - . ^ _ ` | ~` plus
`DIGIT` (10) plus `ALPHA` (52). Everything else is forbidden: all CTLs (0x00–0x1F, 0x7F), SP, and
the 17 delimiters `( ) < > @ , ; : \ " / [ ] ? = { }`, plus every non-ASCII byte.

**(b) `HeaderValue` is a far weaker filter than `token`, and it does NOT sanitize — it rejects
or accepts, byte-exactly.** `http` 1.4.0's validity predicate is `b >= 32 && b != 127 || b == b'\t'`.
So of the characters the story's rule wrongly permits:

| Wrongly-permitted class | `HeaderValue::try_from` outcome | Net effect |
|---|---|---|
| `\n` `\r` `\0` and all other CTLs except TAB | **REJECTED** → `InvalidHeaderValue` | deferred, opaque runtime error (see (c)) |
| TAB (0x09) | **ACCEPTED** | silently emits a malformed cookie-name on the wire |
| SP (0x20) | **ACCEPTED** | silently emits a malformed cookie-name on the wire |
| `( ) < > @ , ; \ " / [ ] ? = { }` (16 delimiters; `:` already blocked by the existing rule) | **ACCEPTED** | silently emits a malformed cookie-name; **`;` and `=` permit cookie-pair injection into the same `Cookie` header** |
| non-ASCII (UTF-8 continuation bytes 0x80–0xFF) | **ACCEPTED** | silently emits non-conformant bytes |

There is **no sanitization anywhere** in the path. `try_from_generic` iterates bytes and either
returns `Err` on the first invalid byte or stores the input verbatim — it never strips, escapes, or
replaces.

**(c) `reqwest::RequestBuilder::header()` DOES store the error internally and defer it.** Confirmed
from source: `header()` returns `Self` (not `Result`); on conversion failure it assigns
`self.request = Err(crate::error::builder(e.into()))`. That `Err` surfaces at:
- `RequestBuilder::build() -> crate::Result<Request>` — `build()` is literally `self.request`;
- `RequestBuilder::send()` — `match self.request { Ok(req) => ..., Err(err) => Pending::new_err(err) }`,
  i.e. the returned future resolves immediately to `Err`.

**And the surfaced message is maximally opaque.** `reqwest::Error`'s `Display` for
`Kind::Builder` writes the literal string `"builder error"` and nothing else. It appends
`" for url (...)"` only when a URL was attached, and the `header()` path attaches none. The real
cause — `"failed to parse header value"` — is reachable *only* via `StdError::source()`. So
`format!("{e}")` on this failure yields exactly:

```
builder error
```

Confirming the story's stated fear precisely: a sensor spec containing `\n` in a cookie name loads
cleanly at boot and then fails **every** query at request-build time with `builder error`.

### EVIDENCE

**RFC 6265 / RFC 9110 grammar** (verified against `rfc-editor.org`):

RFC 9110 §5.6.2 "Tokens" — quoted verbatim: *"A token is a sequence of characters that are all from
the set of visible US-ASCII characters, excluding delimiters."*

```abnf
token          = 1*tchar
tchar          = "!" / "#" / "$" / "%" / "&" / "'" / "*"
               / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
               / DIGIT / ALPHA
```

- <https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.2>
- RFC 6265 §4.1.1 defines `cookie-pair = cookie-name "=" cookie-value` with `cookie-name = token`,
  delegating `token` to RFC 2616 §2.2: <https://www.rfc-editor.org/rfc/rfc6265.html#section-4.1.1>
- RFC 7230 §3.2.6 `tchar` is **textually identical** to RFC 9110 §5.6.2; RFC 9110 obsoletes RFC 7230.
  <https://www.rfc-editor.org/rfc/rfc7230.html#section-3.2.6>
- RFC 2616 §2.2 (the spec RFC 6265 actually cites) expresses the same set complementarily:
  `token = 1*<any CHAR except CTLs or separators>` with
  `separators = "(" | ")" | "<" | ">" | "@" | "," | ";" | ":" | "\" | <"> | "/" | "[" | "]" | "?" | "=" | "{" | "}" | SP | HT`.
  The two definitions coincide: 94 printable non-SP ASCII chars − 17 printable separators = **77**,
  matching the `tchar` enumeration exactly. <https://www.rfc-editor.org/rfc/rfc2616.html#section-2.2>
- RFC 6265bis (`draft-ietf-httpbis-rfc6265bis`) requires cookie octets be processed as US-ASCII and
  mandates rejecting control characters in cookie-name and cookie-value, while acknowledging that
  historical servers and user agents have been lax about the strict `token` grammar.
  <https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-rfc6265bis>

**`http` 1.4.0 `HeaderValue` — SOURCE READ**
(`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/http-1.4.0/src/header/value.rs`):

```rust
// value.rs:556-559 — the ONLY validity gate
#[inline]
fn is_valid(b: u8) -> bool {
    b >= 32 && b != 127 || b == b'\t'
}
```

```rust
// value.rs:212-225 — reject-or-store-verbatim; no sanitization path exists
fn try_from_generic<T: AsRef<[u8]>, F: FnOnce(T) -> Bytes>(
    src: T, into: F,
) -> Result<HeaderValue, InvalidHeaderValue> {
    for &b in src.as_ref() {
        if !is_valid(b) {
            return Err(InvalidHeaderValue { _priv: () });
        }
    }
    Ok(HeaderValue { inner: into(src), is_sensitive: false })
}
```

`from_str` (`value.rs:105`), `from_bytes` (`value.rs:151`), `from_maybe_shared` (`value.rs:159`),
and every `TryFrom` impl (`&str` `value.rs:498`, `&String` `:506`, `&[u8]` `:515`, `String` `:524`,
`Vec<u8>` `:533`) all funnel through `try_from_generic`. The crate's own doctests assert the
behaviour both ways: `HeaderValue::from_str("\n")` → `is_err()` (`value.rs:100-101`), while
`HeaderValue::from_bytes(b"hello\xfa")` → `.unwrap()` succeeds (`value.rs:139-140`), proving high
bytes are accepted. The `from_bytes` doc comment states it directly: *"Only byte values between 32
and 255 (inclusive) are permitted, excluding byte 127 (DEL)."* (`value.rs:129-130`).

Note the asymmetry with `HeaderValue::to_str()`, which uses the *stricter*
`is_visible_ascii(b) = b >= 32 && b < 127 || b == b'\t'` (`value.rs:552-554`) and therefore
`Err(ToStrError)`s on a value that was legally constructed from high bytes. A header value can be
constructible and un-`to_str`-able.

Docs: <https://docs.rs/http/1.4.0/http/header/struct.HeaderValue.html>

**`reqwest` 0.12.28 deferred error — SOURCE READ**
(`~/.cargo/registry/.../reqwest-0.12.28/src/async_impl/request.rs`):

```rust
// request.rs:194-202
pub fn header<K, V>(self, key: K, value: V) -> RequestBuilder
where HeaderName: TryFrom<K>, <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
      HeaderValue: TryFrom<V>, <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
{ self.header_sensitive(key, value, false) }

// request.rs:212-233 — the deferral
let mut error = None;
if let Ok(ref mut req) = self.request {
    match <HeaderName as TryFrom<K>>::try_from(key) {
        Ok(key) => match <HeaderValue as TryFrom<V>>::try_from(value) {
            Ok(mut value) => { ...; req.headers_mut().append(key, value); }
            Err(e) => error = Some(crate::error::builder(e.into())),
        },
        Err(e) => error = Some(crate::error::builder(e.into())),
    };
}
if let Some(err) = error { self.request = Err(err); }
self
```

```rust
// request.rs:482-484
pub fn build(self) -> crate::Result<Request> { self.request }

// request.rs:516-521
pub fn send(self) -> impl Future<Output = Result<Response, crate::Error>> {
    match self.request {
        Ok(req) => self.client.execute_request(req),
        Err(err) => Pending::new_err(err),
    }
}
```

```rust
// reqwest-0.12.28/src/error.rs:227-272 — the opaque Display
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner.kind {
            Kind::Builder => f.write_str("builder error")?,
            ...
        };
        if let Some(url) = &self.inner.url { write!(f, " for url ({url})")?; }
        Ok(())
    }
}
// error.rs:275-279 — cause only via source()
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> { self.inner.source.as_ref().map(...) }
}
```

The eventual root-cause string is `http`'s
`impl Display for InvalidHeaderValue { ... "failed to parse header value" }`
(`http-1.4.0/src/header/value.rs:569-573`), reached through `http::Error`'s transparent
`Display::fmt(self.get_ref(), f)` (`http-1.4.0/src/error.rs:43-47`). Two `source()` hops from the
top-level message.

Also worth noting: because `header_sensitive` guards with `if let Ok(ref mut req) = self.request`,
the **first** error wins and all subsequent `.header()` calls become no-ops. If several cookie names
are invalid you get one error, for whichever failed first.

Docs: <https://docs.rs/reqwest/0.12.28/reqwest/struct.RequestBuilder.html#method.header>

### VERSION-SPECIFICITY

- `is_valid` is byte-identical in the two `http` 1.4.x versions present on this machine (1.4.0 locked,
  1.4.1 also vendored). This predicate has been stable across `http` 0.2 and all of 1.x; it is not a
  version-fragile claim.
- `reqwest`'s store-the-error-in-the-builder pattern has been present since well before 0.12 and is
  the documented contract, not an implementation accident. Verified at the locked 0.12.28.
- RFC 9110 (June 2022) obsoletes RFC 7230; the `token`/`tchar` production is unchanged between them,
  so citing either is safe. RFC 6265's own reference is to the older RFC 2616, whose set coincides.

### IMPLICATION FOR IMPLEMENTATION

Tighten the cookie-name rule from "non-empty, no colon" to **"non-empty and every byte is a
`tchar`"** — i.e. `name.chars().all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c))`
— and reject at spec-load time with a specific `E-SENSOR-NNN` error naming the offending character
and byte offset. The colon-only rule is not merely incomplete: for `;`, `=`, SP, TAB and high bytes
it is **silently exploitable** (`;` allows injecting extra cookie pairs into the same `Cookie`
header) because `HeaderValue` accepts those bytes, and for CTLs it produces a boot-clean spec that
fails every query at `.build()`/`.send()` with the literally-uninformative message `builder error`.
Add a wire-shape test per CLAUDE.md wire-shape assertion discipline asserting the serialized
`Cookie` header bytes for a name containing `;`, and a negative test asserting the spec-load
rejection (not the request-build rejection) for a name containing `\n`.

---

## RQ-2 (MEDIUM, expected confirm-only) — edition-2024 dyn-compatibility of defaulted `Pin<Box<dyn Future>>` trait methods

### QUESTION

Is a defaulted trait method with signature
`fn get_token<'a>(&'a self, spec: &'a SensorSpec, client_id: &'a OrgSlug) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>>`
— body delegating to `self.acquire_token(spec, client_id)` — dyn-compatible on current stable Rust
with the trait used as `Arc<dyn AuthProvider>`? Did edition 2024 change anything? Any
lifetime-elision or variance gotcha?

### ANSWER

**Confirm-only, as expected. Yes, fully dyn-compatible. Edition 2024 changes nothing here. No
lifetime or variance gotcha.**

1. **Provided (defaulted) bodies are irrelevant to dyn-compatibility.** The Rust Reference's
   dyn-compatibility rules are stated purely over *signatures*; the word "default" does not appear
   in them. Your signature satisfies every condition: receiver is `&self`; no method-level *type*
   parameters (lifetime parameters are explicitly allowed); `Self` appears only in the receiver
   type; the return type is a **concrete** `Pin<Box<dyn Future + Send + 'a>>`, not an opaque type;
   and there is no `where Self: Sized` bound. `Arc<Self>` is on the allowed-receiver list too, so
   `Arc<dyn AuthProvider>` dispatch is fine.

2. **Edition 2024 introduced no dyn-compatibility rule change.** The edition-2024 items that touch
   trait objects are (a) bare trait objects becoming a hard error — you already write `dyn`, so
   moot; (b) **RFC 3498** RPIT lifetime capture, which applies to *return-position `impl Trait`*
   and `async fn`; and (c) the `gen` keyword reservation. `Pin<Box<dyn Future<...> + Send + 'a>>` is
   a concrete named type, **not** `impl Trait` and **not** `async fn`, so RFC 3498 does not touch it.
   The hand-rolled-boxed-future pattern is precisely the pattern that *avoids* both the opaque-return
   dyn-incompatibility and the RPIT capture rules. Nothing to migrate.

3. **The "object safety" → "dyn compatibility" rename is terminology only, with zero semantic
   change.** It was **not** an RFC — it went through the lang-team design-meeting/MCP process
   (`rust-lang/lang-team` issue #286), tracked at `rust-lang/rust` issue #130852. Compiler and
   library wording shipped in **Rust 1.83.0**; Reference and rustdoc wording followed in **1.84.0**.
   The Reference now carries the bare note: *"This concept was formerly known as object safety."*
   **RQ-2's suspicion that an "accompanying semantic change" exists is unfounded** — there is none.

4. **No lifetime-elision or variance gotcha for the delegating default body.** Because both methods
   declare the same explicit `<'a>` and tie `&'a self` to the returned `+ 'a` boxed future, the
   delegation `self.acquire_token(spec, client_id)` returns exactly the declared return type — no
   coercion, no reborrow, no variance question. Explicit `'a` on both is what makes this trivially
   sound; the failure mode people hit is *omitting* the `'a` on the boxed trait object (which then
   defaults to `+ 'static` and cannot hold a `&'a self` borrow). You are not doing that. And because
   the default body calls only `&self` methods and never constructs `Self`, no `Self: Sized` bound
   is needed — adding one would in fact make the method explicitly non-dispatchable and unavailable
   through `Arc<dyn AuthProvider>`.

### EVIDENCE

Rust Reference, "Dyn compatibility" (fetched 2026-07-24), quoted verbatim in the load-bearing part:

> A trait is *dyn compatible* if it has the following qualities:
> - All supertraits must also be dyn compatible.
> - `Sized` must not be a supertrait. In other words, it must not require `Self: Sized`.
> - It must not have any associated constants.
> - It must not have any associated types with generics.
> - All associated functions must either be dispatchable from a trait object or be explicitly non-dispatchable:
>   - Dispatchable functions must:
>     - Not have any type parameters (although lifetime parameters are allowed).
>     - Be a method that does not use `Self` except in the type of the receiver.
>     - Have a receiver with one of the following types: `&Self` (i.e. `&self`), `&mut Self` (i.e. `&mut self`), `Box<Self>`, `Rc<Self>`, `Arc<Self>`, `Pin<P>` where `P` is one of the types above
>     - Not have an opaque return type; that is,
>       - Not be an `async fn` (which has a hidden `Future` type).
>       - Not have a return position `impl Trait` type (`fn example(&self) -> impl Trait`).
>     - Not have a `where Self: Sized` bound (receiver type of `Self` (i.e. `self`) implies this).
>   - Explicitly non-dispatchable functions require:
>     - Have a `where Self: Sized` bound (receiver type of `Self` (i.e. `self`) implies this).
> - The `AsyncFn`, `AsyncFnMut`, and `AsyncFnOnce` traits are not dyn-compatible.
>
> Note: This concept was formerly known as *object safety*.

Confirmed by direct fetch: <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>.
**Default bodies are not mentioned anywhere in the list** — this is the affirmative textual basis
for answer (1). Note the list's own carve-out "*although lifetime parameters are allowed*", which is
the explicit sanction for your `<'a>`.

- `E0038` (Rust Error Index) enumerates the same conditions as the reasons a trait cannot be made
  into an object: <https://doc.rust-lang.org/error_codes/E0038.html>
- Edition 2024 change inventory (nothing on dyn-compatibility):
  <https://doc.rust-lang.org/edition-guide/rust-2024/index.html>
- RFC 3498 "lifetime capture rules 2024" — scoped to RPIT / `impl Trait`, not `Box<dyn Trait>`:
  <https://rust-lang.github.io/rfcs/3498-lifetime-capture-rules-2024.html>
- Rename provenance: `rust-lang/lang-team` issue #286 (design-meeting proposal, reached FCP
  disposition-merge) <https://github.com/rust-lang/lang-team/issues/286>; tracking issue
  `rust-lang/rust` #130852 <https://github.com/rust-lang/rust/issues/130852>, which states
  "Changes to the compiler & the libraries will be included in 1.83. Changes to the Reference &
  rustdoc will be included in 1.84."; library rename PR `rust-lang/rust` #130827 carries the
  **1.83.0** milestone <https://github.com/rust-lang/rust/pull/130827>.

**Correction to the RQ-2 framing:** the question hypothesises "RFC 3782 / RFC 3729" for the rename.
A targeted search for an RFC-numbered rename returned no such RFC; the change went through the
lang-team process instead. Do not cite an RFC number for it in story text.

### VERSION-SPECIFICITY

- Rules verified against the current Rust Reference as of 2026-07-24. prism pins its toolchain via
  `rust-toolchain.toml` (stable), which is ≫ 1.84, so both the rules and the new terminology apply.
- The dyn-compatibility conditions have been stable in substance since RFC 0255 (2014); the only
  post-1.0 *relaxations* have been additive (e.g. Rust 1.87 permitting omission of an uncallable
  method). Nothing has been made *stricter* in a way that could break this trait.

### IMPLICATION FOR IMPLEMENTATION

Add `get_token` with the defaulted delegating body as planned — no `#[async_trait]`, no
`where Self: Sized`, and keep the explicit `<'a>` on **both** methods with `+ Send + 'a` on the
boxed future. Do **not** add a `Self: Sized` bound "to be safe": that would make the method
explicitly non-dispatchable and thus uncallable through `Arc<dyn AuthProvider>`, which is the exact
usage site. No edition-2024 migration work, no ADR needed.

---

## RQ-3 (MEDIUM) — chrono 0.4.44 `FromStr` leniency

### QUESTION

A sibling story is premised on "lenient chrono `FromStr` parsing" covering RFC-3339 **and**
ISO-8601-without-timezone **and** bare Unix-epoch integers from a single `FromStr` call. Is that
premise true? What does each of `DateTime<Utc>`, `DateTime<FixedOffset>`, `NaiveDateTime` accept and
reject? Which of `parse_from_rfc3339` / `parse_from_str` / `FromStr` / `DateTime::from_timestamp`
are deprecated or behaviour-changed?

### ANSWER

**The sibling story's premise is FALSE. No single chrono `FromStr` impl accepts all three forms —
not even two of the three. A multi-format parser must be hand-written.**

Acceptance matrix, verified by reading chrono 0.4.44 source:

| Input form | `DateTime<Utc>` | `DateTime<FixedOffset>` | `NaiveDateTime` |
|---|---|---|---|
| `2026-07-24T12:00:00Z` (RFC-3339, `Z`) | ACCEPT | ACCEPT | **REJECT** (`TOO_LONG` — trailing `Z`) |
| `2026-07-24T12:00:00+02:00` (RFC-3339, offset) | ACCEPT | ACCEPT | **REJECT** (`TOO_LONG`) |
| `2026-07-24 12:00:00+0200` (space sep, no colon in offset) | ACCEPT | ACCEPT | **REJECT** |
| `2026-07-24T12:00:00` (ISO-8601, **no timezone**) | **REJECT** (`TOO_SHORT`) | **REJECT** (`TOO_SHORT`) | ACCEPT |
| `2026-07-24 12:00:00` (no tz, **space** separator) | REJECT | REJECT | **REJECT** (`INVALID` — `T` is a hard literal) |
| `1785240000` (bare epoch seconds) | **REJECT** | **REJECT** | **REJECT** |

Three consequences the story must absorb:

1. **`DateTime<Utc>` / `DateTime<FixedOffset>` require an offset.** Their leniency is real but
   narrow: it is *relaxed RFC 3339* — space-or-`T` separator, unpadded components, interior spaces,
   optional colon in the offset, `UTC` accepted as an offset spelling. It is **not** "timezone
   optional." A timezone-less ISO-8601 string is `TOO_SHORT`, not defaulted-to-UTC.
2. **`NaiveDateTime` is *stricter*, not more lenient, on the separator.** Its item list contains
   `Item::Literal("T")`, an exact-match literal. `"2026-07-24 12:00:00".parse::<NaiveDateTime>()`
   **fails** with `INVALID`. This is a common and costly misconception — the `DateTime` impls accept
   a space, the `NaiveDateTime` impl does not.
3. **Bare epoch integers are accepted by NO `FromStr` impl.** Every impl starts with a `Year`
   numeric item followed by a hard `-` literal, so `"1785240000"` cannot parse. Epoch ingestion
   requires `s.parse::<i64>()` then `DateTime::from_timestamp_secs` / `from_timestamp`.

**Deprecation status in 0.4.44** (all verified by source read):

| API | Status in 0.4.44 |
|---|---|
| `DateTime::parse_from_rfc3339` | **NOT deprecated** (`datetime/mod.rs:1071`) |
| `DateTime::parse_from_str` | **NOT deprecated** (`datetime/mod.rs:1104`) |
| `FromStr` impls for `DateTime<Utc>` / `DateTime<FixedOffset>` / `DateTime<Local>` / `NaiveDateTime` | **NOT deprecated** |
| `DateTime::from_timestamp(secs, nsecs) -> Option<Self>` | **NOT deprecated**, `const` (`datetime/mod.rs:803`) |
| `DateTime::from_timestamp_secs(secs) -> Option<Self>` | **NOT deprecated**, `const` (`datetime/mod.rs:768`) — the ergonomic single-arg wrapper; prefer this for epoch-seconds |
| `DateTime::from_timestamp_millis` / `_micros` / `_nanos` | **NOT deprecated** (`:838`, `:875`, `:910`) |
| `NaiveDateTime::from_timestamp` | **DEPRECATED since 0.4.23** → use `DateTime::from_timestamp` |
| `NaiveDateTime::from_timestamp_opt` / `_millis` / `_micros` / `_nanos` | **DEPRECATED since 0.4.35** → use the `DateTime::*` equivalents |
| `TimeZone::timestamp(secs, nsecs)` | **DEPRECATED since 0.4.23** → `timestamp_opt()` |
| `TimeZone::timestamp_millis` | **DEPRECATED since 0.4.23** → `timestamp_millis_opt()` |

The trap: the naive-typed epoch constructors are exactly the ones that are deprecated. Any code
reaching for `NaiveDateTime::from_timestamp*` will emit deprecation warnings, which `just clippy`
(`-D warnings`) turns into a build failure.

### EVIDENCE

All from `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chrono-0.4.44/`.

**`DateTime<Utc>` delegates to `DateTime<FixedOffset>`** (`src/datetime/mod.rs:1889-1895`):

```rust
impl str::FromStr for DateTime<Utc> {
    type Err = ParseError;
    fn from_str(s: &str) -> ParseResult<DateTime<Utc>> {
        s.parse::<DateTime<FixedOffset>>().map(|dt| dt.with_timezone(&Utc))
    }
}
```

`DateTime<Local>` does the same (`:1911-1917`). So all three share one grammar.

**`DateTime<FixedOffset>` uses the relaxed parser, and the offset is mandatory**
(`src/format/parse.rs:585-596`, and `parse_rfc3339_relaxed` at `:598-646`):

```rust
impl str::FromStr for DateTime<FixedOffset> {
    type Err = ParseError;
    fn from_str(s: &str) -> ParseResult<DateTime<FixedOffset>> {
        let mut parsed = Parsed::new();
        let (s, _) = parse_rfc3339_relaxed(&mut parsed, s)?;
        if !s.trim_start().is_empty() { return Err(TOO_LONG); }
        parsed.to_datetime()
    }
}
```

```rust
/// Accepts a relaxed form of RFC3339.
/// Differences with RFC3339:
/// - Values don't require padding to two digits.
/// - Years outside the range 0...=9999 are accepted, but they must include a sign.
/// - `UTC` is accepted as a valid timezone name/offset (...)
/// - There can be spaces between any of the components.
/// - The colon in the offset may be missing.
fn parse_rfc3339_relaxed<'a>(parsed: &mut Parsed, mut s: &'a str) -> ParseResult<(&'a str, ())> {
    ...
    s = match s.as_bytes().first() {
        Some(&b't' | &b'T' | &b' ') => &s[1..],   // space OR 'T' OR 't'
        Some(_) => return Err(INVALID),
        None => return Err(TOO_SHORT),
    };
    s = parse_internal(parsed, s, TIME_ITEMS.iter())?;
    s = s.trim_start();
    let (s, offset) = if s.len() >= 3 && "UTC".as_bytes().eq_ignore_ascii_case(&s.as_bytes()[..3]) {
        (&s[3..], 0)
    } else {
        scan::timezone_offset(s, scan::colon_or_space, true, false, true)?   // <-- `?` = MANDATORY
    };
    parsed.set_offset(i64::from(offset))?;
    Ok((s, ()))
}
```

The `?` on `scan::timezone_offset` is the whole answer to "is the timezone optional": **no**. On an
empty remainder that call returns `Err(TOO_SHORT)`. `scan::timezone_offset` accepts `Z`/`z` (via
`allow_zulu = true`, `src/format/scan.rs:213-217`), `+`/`-`, and even U+2212 MINUS SIGN (via
`allow_tz_minus_sign = true`, `scan.rs:236-244`) — but something must be there.

**`NaiveDateTime` requires a literal `T` and forbids any trailing offset**
(`src/naive/datetime/mod.rs:2131-2159`):

```rust
impl str::FromStr for NaiveDateTime {
    type Err = ParseError;
    fn from_str(s: &str) -> ParseResult<NaiveDateTime> {
        const ITEMS: &[Item<'static>] = &[
            Item::Numeric(Numeric::Year, Pad::Zero), Item::Space(""), Item::Literal("-"),
            Item::Numeric(Numeric::Month, Pad::Zero), Item::Space(""), Item::Literal("-"),
            Item::Numeric(Numeric::Day, Pad::Zero), Item::Space(""),
            Item::Literal("T"),   // XXX shouldn't this be case-insensitive?
            Item::Numeric(Numeric::Hour, Pad::Zero), Item::Space(""), Item::Literal(":"),
            Item::Numeric(Numeric::Minute, Pad::Zero), Item::Space(""), Item::Literal(":"),
            Item::Numeric(Numeric::Second, Pad::Zero), Item::Fixed(Fixed::Nanosecond),
            Item::Space(""),
        ];
        let mut parsed = Parsed::new();
        parse(&mut parsed, s, ITEMS.iter())?;
        parsed.to_naive_datetime_with_offset(0)
    }
}
```

Two mechanics settle the two rejections:

```rust
// src/format/parse.rs:346-354 — Literal is exact-match, hence space separator => INVALID
Item::Literal(prefix) => {
    if s.len() < prefix.len() { return Err(TOO_SHORT); }
    if !s.starts_with(prefix) { return Err(INVALID); }
    s = &s[prefix.len()..];
}
// src/format/parse.rs:367-369 — Space("") only trims whitespace; it cannot absorb a 'Z' or '+02:00'
Item::Space(_) => { s = s.trim_start(); }
// src/format/parse.rs:285-295 — any remaining chars are a hard error
pub fn parse<...>(parsed: &mut Parsed, s: &str, items: I) -> ParseResult<()> {
    match parse_internal(parsed, s, items) {
        Ok("") => Ok(()),
        Ok(_) => Err(TOO_LONG),   // trailing 'Z' / '+02:00' lands here
        Err(e) => Err(e),
    }
}
```

The chrono doc comment on `DateTime::parse_from_str` states the same constraint from the other
direction: *"Note that this method requires a timezone in the input string. See
`NaiveDateTime::parse_from_str` for a version that does not require a timezone."*
(<https://docs.rs/chrono/0.4.44/chrono/struct.DateTime.html#method.parse_from_str>)

Deprecation attributes read directly: `naive/datetime/mod.rs:119` (`from_timestamp`, since 0.4.23),
`:136` / `:151` / `:168` / `:191` (`from_timestamp_millis` / `_micros` / `_nanos` /
`from_timestamp_opt`, all since 0.4.35), `offset/mod.rs:441` / `:478` (`TimeZone::timestamp`,
`timestamp_millis`, since 0.4.23).

### VERSION-SPECIFICITY

Everything above is a source read at **exactly 0.4.44**, the locked version — not inferred from docs
of another release.

**INCONCLUSIVE (two sub-items, neither affecting the decision):**
- The release in which `DateTime::from_timestamp_secs` was introduced could not be established.
  `chrono`'s `CHANGELOG.md` explicitly covers only *"changes up to and including version 0.4.19"*
  and defers later releases to GitHub Releases (<https://github.com/chronotope/chrono/blob/master/CHANGELOG.md>),
  and the method does not appear in the release notes I could retrieve. A `perplexity_ask` reply
  asserting "0.4.31" is **rejected as unreliable** — it conflicts with the 0.4.31 release notes,
  which record `from_timestamp`, not `from_timestamp_secs`. Its **presence and non-deprecation in
  0.4.44 are directly source-verified**, which is what the story needs.
- Likewise the release in which `FromStr for DateTime<FixedOffset>` switched to
  `parse_rfc3339_relaxed` could not be pinned. A `perplexity_ask` reply asserting 0.4.35 is
  **rejected**: the v0.4.35 release notes I fetched contain no mention of `FromStr`, RFC 3339, or
  relaxed parsing. Behaviour in 0.4.44 is source-verified.

Do not put either introduction version into story text. Cite the 0.4.44 source anchor instead
(TD-VSDD-091: cite function names and behavioural anchors, not decayed line pins — the line numbers
here are provided as a reading aid for this pass, the function names are the durable anchors).

### IMPLICATION FOR IMPLEMENTATION

Reject the sibling story's "lenient `FromStr` covers all three" premise and route a correction to
its owner. Hand-write an explicit ordered multi-format parser: **(1)** try
`s.parse::<DateTime<FixedOffset>>()` (covers RFC-3339 and the relaxed variants, offset present);
**(2)** on failure try `s.parse::<NaiveDateTime>()` **plus** a `parse_from_str` fallback with
`"%Y-%m-%d %H:%M:%S"` to cover the space-separated no-timezone form the `NaiveDateTime` `FromStr`
rejects, then apply a documented default-timezone policy (a real product decision — do not let it
be implicit); **(3)** on failure try `s.parse::<i64>()` → `DateTime::from_timestamp_secs`. Use
`DateTime::from_timestamp_secs` / `DateTime::from_timestamp*`, never
`NaiveDateTime::from_timestamp*` (deprecated → `-D warnings` build failure). Table-drive the
negative cases: the six REJECT cells above should each have a test.

---

## RQ-4 (LOW) — serde `#[serde(default)]` on `Option<T>`

### QUESTION

In serde 1.0.228, for a self-describing format with no null literal (TOML): is `#[serde(default)]`
on an `Option<T>` field identical to omitting the attribute? Does it interact with
`deny_unknown_fields` or `flatten`? Is there a Serialize-side asymmetry — does `toml` 0.8.23 skip
`None` struct fields or error?

### ANSWER

**Your reading of the derive is correct.** For a plain `Option<T>` field with **no other
field-level attributes**, `#[serde(default)]` is **observably redundant** — both paths yield `None`
on an absent field. Confirmed by reading both `serde_derive` 1.0.228 and `serde` 1.0.228.

But there are **three qualifications**, and the first is a real trap:

1. **`#[serde(default)]` becomes load-bearing the moment `#[serde(deserialize_with = "...")]` is
   added to the same field.** The derive has two distinct absent-field codegen branches. Without
   `deserialize_with`, absence routes through `missing_field(name)?` → `deserialize_option` →
   `visit_none()` → `Ok(None)`. **With** `deserialize_with` and **no** default, the derive emits a
   hard `return Err(missing_field(name))` — the `Option<T>` field becomes **mandatory**. So the
   attribute is redundant *today* and becomes mandatory the day someone adds a custom deserializer.
   This is the single most important finding in RQ-4.

2. **No interaction with `deny_unknown_fields`.** `deny_unknown_fields` is a container attribute
   governing *unknown* keys; `default` governs *absent known* keys. They are orthogonal and compose
   without surprise. (The documented `deny_unknown_fields` incompatibility is with **`flatten`**,
   not with `default`.)

3. **`flatten` on the same struct changes the deserialization path but not the `Option` outcome.**
   Adding `flatten` switches the struct to a buffered content-based path
   (`__private::de::FlatMapDeserializer`) and disables `deny_unknown_fields`. Absent `Option<T>`
   fields still resolve to `None`. Flagged **MEDIUM confidence** — I did not read the
   `FlatMapDeserializer` absent-field branch, and this is not currently load-bearing for the story.

**Serialize side — no asymmetry that requires action for struct fields.** `toml` 0.8.23 delegates
serialization to `toml_edit` 0.22.27, which **silently SKIPS `None`-valued struct fields and map
values**. It does this via a sentinel-error dance, not a pre-check: `serialize_none` sets an
out-param flag and returns `Err(Error::UnsupportedNone)`; the struct/map field driver then swallows
exactly that error when the flag is set. So **`#[serde(skip_serializing_if = "Option::is_none")]`
is NOT required for struct fields.** It *is* required — or rather, `None` is a hard error — in two
other positions: a bare top-level `None`, and `None` **inside a sequence** (`Vec<Option<T>>`), where
the sentinel is never swallowed and the error surfaces as `unsupported None value`.

### EVIDENCE

**Derive codegen — SOURCE READ**
(`~/.cargo/registry/.../serde_derive-1.0.228/src/de.rs:763-803`, `fn expr_is_missing`):

```rust
fn expr_is_missing(field: &Field, cattrs: &attr::Container) -> Fragment {
    match field.attrs.default() {
        attr::Default::Default => {                                  // #[serde(default)]
            let func = quote_spanned!(span=> _serde::#private::Default::default);
            return quote_expr!(#func());                             // => Option::default() == None
        }
        attr::Default::Path(path) => { return Fragment::Expr(quote_spanned!(...=> #path())); }
        attr::Default::None => { /* below */ }
    }
    match *cattrs.default() { /* container-level default: __default.#member */ }

    let name = field.attrs.name().deserialize_name();
    match field.attrs.deserialize_with() {
        None => {                                                    // no attribute at all
            let func = quote_spanned!(span=> _serde::#private::de::missing_field);
            quote_expr! { #func(#name)? }                            // => Ok(None) for Option<T>
        }
        Some(_) => {                                                 // deserialize_with, no default
            quote_expr! {
                return _serde::#private::Err(
                    <__A::Error as _serde::de::Error>::missing_field(#name))   // HARD ERROR
            }
        }
    }
}
```

That final `Some(_)` arm is the proof of qualification (1).

**`missing_field` — SOURCE READ** (`~/.cargo/registry/.../serde-1.0.228/src/private/de.rs:24-61`):

```rust
pub fn missing_field<'de, V, E>(field: &'static str) -> Result<V, E>
where V: Deserialize<'de>, E: Error,
{
    struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);

    impl<'de, E> Deserializer<'de> for MissingFieldDeserializer<E> where E: Error {
        type Error = E;
        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, E> where V: Visitor<'de> {
            Err(Error::missing_field(self.0))
        }
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E> where V: Visitor<'de> {
            visitor.visit_none()                                     // <-- the Option escape hatch
        }
        serde_core::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }
    let deserializer = MissingFieldDeserializer(field, PhantomData);
    Deserialize::deserialize(deserializer)
}
```

`deserialize_option` is the *sole* method not forwarded to the erroring `deserialize_any`. Exactly
as RQ-4 hypothesised. (Incidental observation for future reference: at 1.0.228 the forwarding macro
comes from the split-out `serde_core` crate — a `serde` internals refactor, no behavioural effect.)

Docs corroboration: <https://serde.rs/field-attrs.html#default>,
<https://serde.rs/container-attrs.html#deny_unknown_fields>,
<https://serde.rs/attr-flatten.html> (which states the `flatten` ↔ `deny_unknown_fields`
incompatibility).

**`toml` → `toml_edit` delegation — SOURCE READ** (`toml-0.8.23/Cargo.toml:99-101, 153-157`):
`display = ["dep:toml_edit", "toml_edit?/display"]`, `toml_edit = { version = "0.22.27",
features = ["serde"], optional = true, default-features = false }`.

**`None` is skipped for struct fields / map values — SOURCE READ**
(`toml_edit-0.22.27/src/ser/map.rs`):

```rust
// map.rs:470-473 — sentinel: flag + error
fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    *self.is_none = true;
    Err(Error::unsupported_none())
}

// map.rs:196-215 (SerializeStruct::serialize_field) — sentinel swallowed
let mut is_none = false;
let value_serializer = MapValueSerializer::new(&mut is_none);
let res = value.serialize(value_serializer);
match res {
    Ok(item) => { self.items.insert(crate::Key::new(key), crate::Item::Value(item)); }
    Err(e) => {
        if !(e == Error::unsupported_none() && is_none) { return Err(e); }
        // else: field silently omitted
    }
};
Ok(())
```

The identical swallow appears in `SerializeMap::serialize_value` (`map.rs:165-185`).

**`None` is a hard error elsewhere — SOURCE READ.** `toml_edit-0.22.27/src/ser/value.rs:147-149`
has `fn serialize_none(self) -> ... { Err(Error::unsupported_none()) }` with no sentinel flag, and
`src/ser/array.rs` contains **no** `is_none` handling at all (grep for `is_none|unsupported_none`
returns no matches) — so a `None` element in a sequence propagates the error. The error variant and
its message are `Error::UnsupportedNone` → `"unsupported None value"`
(`toml_edit-0.22.27/src/ser/mod.rs:114`, `:140`, `:168`).

### VERSION-SPECIFICITY

- `expr_is_missing` and `missing_field` read at exactly serde/serde_derive **1.0.228**. This shape
  has been stable across serde 1.x; the `deserialize_with`-without-default hard-error branch is
  long-standing, not new.
- The `toml_edit` sentinel-swallow was read at exactly **0.22.27**, the version `toml` 0.8.23 pins.
  **Do not port this conclusion to `toml` 0.9.x** (also in the lockfile at 0.9.12+spec-1.1.0) — 0.9
  restructured serialization and I did not verify it. If a story consumes the `toml` 0.9 edge, the
  skip-vs-error question must be re-verified there.

### IMPLICATION FOR IMPLEMENTATION

Omit `#[serde(default)]` on plain `Option<T>` fields — it is dead attribute noise at 1.0.228 — and
omit `#[serde(skip_serializing_if = "Option::is_none")]` on `Option<T>` **struct fields**, since
`toml_edit` 0.22.27 already skips them. Two guardrails: **(1)** if a story ever adds
`#[serde(deserialize_with = ...)]` to an `Option<T>` field it **must** add `#[serde(default)]` in
the same commit or the field silently becomes mandatory — worth a code-comment at the field and a
round-trip unit test asserting absence yields `None`; **(2)** any `Vec<Option<T>>` or
`Option<T>`-at-top-level serialization target will fail with `unsupported None value` — model those
as `Vec<T>` with the `None`s filtered, not `Vec<Option<T>>`.

---

## RQ-5 (LOW) — thiserror 1.0.69 → 2.x `#[error(...)]` deltas

### QUESTION

Are there breaking changes to `#[error("...")]` attribute syntax, named-field interpolation, or
`Display` generation between 1.0.69 and current 2.x that would affect enum variants being rewritten
now? Would a message template authored today need rework on a future 2.x migration?

### ANSWER

**Ordinary templates are safe. `#[error("bad {name}")]`, `#[error("{0}")]`, `{.0}`/`{.field}`
shorthand, and `#[error(transparent)]` all behave identically in 2.x.** Write templates today
without hedging.

thiserror 2.0.0 has exactly **four** breaking changes. Three touch the `#[error(...)]` surface:

| # | Breaking change | Affects a template authored today? |
|---|---|---|
| 1 | Raw-identifier interpolation `{r#type}` is no longer accepted; use the unraw name `{type}`. (Aligns thiserror with std's formatting macros, which gained implicit argument capture after thiserror 1.x shipped it.) | **Only if a field is named with a Rust keyword** (`r#type`, `r#match`, `r#async`, ...). |
| 2 | Trait bounds are no longer inferred on fields whose value is shadowed by an explicit named argument. Before: `impl<T: Octal> Display for Error<T>`; after: `impl<T> Display for Error<T>` for `#[error("{thing:o}", thing = "...")]`. | **Only if a generic error type shadows a field with an explicit named argument.** Can turn an inferred bound into a missing one → compile error on migration. |
| 3 | Tuple structs / tuple variants may no longer mix numeric `{0}`/`{1}` access with extra positional arguments (ambiguous). | **Only if a tuple variant's template mixes `{0}` with additional positional args.** |
| 4 | Code invoking `derive(Error)` must now have a **direct** dependency on `thiserror`, regardless of the error type's contents. | Not a template concern, but a `Cargo.toml` concern — relevant given both majors are in prism's graph. |

**MSRV: RQ-5's assumed "bump to Rust 1.61" is wrong in both directions.** 1.0.69 declares
`rust-version = "1.61"`; 2.0.18 declares `rust-version = "1.68"`. So 2.x *did* raise MSRV, but to
**1.68**, not 1.61 — 1.61 is the *old* value. (Immaterial for prism: the pinned stable toolchain
far exceeds both.)

`#[error(transparent)]` was not broken; it was **extended** — in 2.x an enum with an enum-level
format message may mark individual variants `transparent` to supersede it. `#[from]`, `#[source]`,
and `#[backtrace]` field inference are unchanged, except for the additive opt-out that lets you name
a field `r#source` so it is *not* treated as `Error::source()`.

### EVIDENCE

thiserror 2.0.0 release notes, retrieved from the release page (2024-11-06) and cross-checked with a
Tavily extraction of the same page. Breaking-changes bullets:

1. *"Referencing keyword-named fields by a raw identifier like `{r#type}` inside a format string is
   no longer accepted; simply use the unraw name like `{type}`"* — with the migration example:
   ```rust
   #[derive(Error, Debug)]
   #[error("... {type} ...")]  // Before: {r#type}
   pub struct Error { pub r#type: Type }
   ```
   *"This aligns thiserror with the standard library's formatting macros, which gained support for
   implicit argument capture later than the release of this feature in thiserror 1.x."*
2. *"Trait bounds are no longer inferred on fields whose value is shadowed by an explicit named
   argument in a format message (#345)"* — with the example:
   ```rust
   // Before: impl<T: Octal> Display for Error<T>
   // After: impl<T> Display for Error<T>
   #[derive(Error, Debug)]
   #[error("{thing:o}", thing = "...")]
   pub struct Error<T> { thing: T }
   ```
3. *"Tuple structs and tuple variants cannot use numerical `{0}` `{1}` access simultaneously with
   extra positional arguments"* (ambiguity).
4. *"Code containing invocations of thiserror's `derive(Error)` must now have a direct dependency on
   the `thiserror` crate"*.

Features (non-breaking): `default-features = false` to drop the std dependency (#373); `r#source`
field-name opt-out (#350); `unconditional_recursion` warning on self-referential Display (#359);
`#[error(fmt = path::to::myfmt)]` out-of-line formatting for enum variants; per-variant
`transparent` superseding an enum-level message.

- <https://github.com/dtolnay/thiserror/releases/tag/2.0.0>
- <https://docs.rs/thiserror/2.0.18/thiserror/> (2.x derive docs still present the same `{field}`
  and `.0` / `.field` shorthand rules as 1.x)
- MSRV: **SOURCE READ** — `thiserror-1.0.69/Cargo.toml:14` → `rust-version = "1.61"`;
  `thiserror-2.0.18/Cargo.toml:14` → `rust-version = "1.68"`.

**Note on the top-level `CHANGELOG.md`:** `raw.githubusercontent.com/dtolnay/thiserror/master/CHANGELOG.md`
returns HTTP 404 — dtolnay's crates use GitHub Releases as the changelog, not a `CHANGELOG.md`.
Cite the release tag URL, not a changelog file.

### VERSION-SPECIFICITY

- Breaking-change list is from the 2.0.0 release notes (2024-11-06). 2.0.1 … 2.0.18 are patch
  releases; no further `#[error(...)]` syntax breakage is documented in the 2.0.x line.
- The three template-affecting changes are all **2.0.0-only**; there is nothing to re-check per
  patch release.

### IMPLICATION FOR IMPLEMENTATION

Author `#[error("...")]` templates today with no hedging, observing three cheap forward-compatible
rules that cost nothing at 1.0.69 and eliminate all migration rework: **(1)** never write
`{r#field}` in a template — use `{field}`, which already works in 1.x; **(2)** for tuple variants,
don't mix `{0}`-style numeric access with extra positional args — prefer named struct-style variants,
which prism's `E-*` taxonomy already favours; **(3)** on any *generic* error type, write explicit
`T: Display`/`T: Octal` bounds rather than relying on thiserror's inference, especially where a
named argument shadows a field. Separately, if a future story moves the first-party dependency edge
to thiserror 2.x, confirm every crate invoking `derive(Error)` declares a direct `thiserror`
dependency (2.0 breaking change #4) — and note both majors are already in `Cargo.lock`.

---

## Summary

| RQ | Answer in one line | Confidence | Implementation impact |
|---|---|---|---|
| **RQ-1** | RFC 6265 `cookie-name` = 77 `tchar` chars only; `HeaderValue` never sanitizes — it **rejects** CTLs (`\n`/`\r`/`\0`) but **accepts** SP, TAB, `;`, `=`, all other delimiters and all high bytes; `reqwest`'s `.header()` defers the error into the builder, surfacing at `.build()`/`.send()` as the literally-opaque `"builder error"`. | **HIGH** (source-read at `http` 1.4.0 + `reqwest` 0.12.28; RFC ABNF quoted verbatim) | **BLOCKING.** Replace the no-colon rule with a full `tchar` whitelist enforced at spec-load. The `;`/`=`/SP gap is a **silent cookie-injection vector**, not just a hygiene issue; the CTL gap yields boot-clean specs failing every query with `builder error`. Add a wire-shape test + a spec-load rejection test. |
| **RQ-2** | Fully dyn-compatible; defaulted bodies are irrelevant to dyn-compatibility; edition 2024 changed nothing (RFC 3498 is RPIT-only, and `Pin<Box<dyn Future>>` is not RPIT); the object-safety→dyn-compatibility rename was **terminology only**, shipped Rust 1.83 compiler / 1.84 docs via lang-team issue #286 (**not** an RFC); no lifetime/variance gotcha with explicit `<'a>` on both methods. | **HIGH** (Rust Reference quoted verbatim; rename provenance traced to tracking issue + milestoned PR) | **NONE — proceed as planned.** Confirm-only, as RQ-2 expected. Keep explicit `<'a>` + `+ Send + 'a` on both methods; do **not** add `where Self: Sized` (it would make the method undispatchable via `Arc<dyn AuthProvider>`). No ADR. |
| **RQ-3** | **The sibling story's premise is FALSE.** No single `FromStr` accepts even two of the three forms: `DateTime<Utc>`/`<FixedOffset>` **require** an offset; `NaiveDateTime` requires a **literal `T`** (rejects space separator) and rejects any offset; **no** impl accepts a bare epoch integer. `parse_from_rfc3339`/`parse_from_str`/`FromStr`/`DateTime::from_timestamp*` are all non-deprecated; **`NaiveDateTime::from_timestamp*` are deprecated** (0.4.23/0.4.35). | **HIGH** on behaviour (source-read at exactly 0.4.44); **INCONCLUSIVE** on two introduction-version attributions | **BLOCKING for the sibling story.** Route a correction to its owner. Hand-write an ordered 3-stage parser (`DateTime<FixedOffset>` → `NaiveDateTime` + `%Y-%m-%d %H:%M:%S` fallback + explicit default-tz policy → `i64` + `from_timestamp_secs`). Use `DateTime::from_timestamp_secs`, never `NaiveDateTime::from_timestamp*` (`-D warnings` build failure). Test all six REJECT cells. |
| **RQ-4** | Your reading is **correct**: `#[serde(default)]` on a plain `Option<T>` is observably redundant (absence → `missing_field` → `deserialize_option` → `visit_none()`). **But** it becomes **mandatory** the moment `#[serde(deserialize_with)]` is added — that branch emits a hard `Err(missing_field)`. Orthogonal to `deny_unknown_fields`. `toml` 0.8.23 (via `toml_edit` 0.22.27) **silently skips** `None` **struct fields**, so `skip_serializing_if` is unnecessary — but `None` **in a sequence or at top level** errors with `unsupported None value`. | **HIGH** on the derive/`missing_field`/toml-skip paths (all source-read); **MEDIUM** on the `flatten` sub-case (not source-verified, not load-bearing) | **LOW, with one guardrail.** Omit the attribute and omit `skip_serializing_if` on `Option<T>` struct fields. Guardrail: any future `#[serde(deserialize_with)]` on an `Option<T>` field must add `#[serde(default)]` in the same commit or the field silently becomes required — worth a field-level comment + an absence-yields-`None` round-trip test. Model `Vec<Option<T>>` as `Vec<T>`. Do not port the toml conclusion to the `toml` 0.9.x edge (unverified). |
| **RQ-5** | Ordinary templates are **safe**: `{name}`, `{0}`, `.0`/`.field`, `transparent` are unchanged in 2.x. Only four 2.0.0 breaking changes, three template-adjacent: `{r#type}` → `{type}`; no bound inference on shadowed fields; no `{0}` mixed with extra positional args; plus a direct-`thiserror`-dependency requirement. **MSRV correction:** 1.0.69 is 1.61; 2.0.18 is **1.68** (RQ-5's "bump to 1.61" inverts it). | **HIGH** (2.0.0 release notes fetched + cross-validated; MSRVs source-read from both `Cargo.toml`s) | **LOW.** Author templates now with three zero-cost forward-compat rules: never `{r#field}`; no `{0}` + extra positionals on tuple variants (prefer named struct-style variants, which prism's `E-*` taxonomy already favours); explicit `T: Display` bounds on generic error types. Note both thiserror majors are already in `Cargo.lock`. |

### Cross-cutting notes for the story-writer

1. **RQ-1 is the only gate that changes the story's acceptance criteria.** RQ-2 is confirm-only.
   RQ-4 and RQ-5 yield authoring guardrails, not scope. RQ-3's blocking impact lands on the
   **sibling** story, which needs a correction routed to its owner before it is materialized.
2. **RQ-3 requires a product decision, not just a parser.** "ISO-8601 without timezone" has no
   defined instant. The default-timezone policy (assume UTC? assume the sensor's configured tz?
   reject?) is a genuine human/product decision under the Canonical Principle's "surface DECISIONS,
   defer no WORK" boundary. Surface it; do not let it become an implicit `Utc` in code.
3. **Two INCONCLUSIVE items are both chrono introduction-version attributions** (RQ-3), neither of
   which affects any implementation choice. Two `perplexity_ask` answers on these were
   **explicitly rejected as unreliable** after they conflicted with the primary sources; they are
   named as rejected in the RQ-3 VERSION-SPECIFICITY note so no downstream reader re-imports them.
4. **Line numbers in this artifact are reading aids, not durable anchors** (TD-VSDD-091). The
   durable anchors are the function/item names: `http::header::value::is_valid`,
   `try_from_generic`; `reqwest::async_impl::request::RequestBuilder::header_sensitive`;
   `chrono::format::parse::parse_rfc3339_relaxed`, `impl FromStr for NaiveDateTime`;
   `serde_derive::de::expr_is_missing`, `serde::private::de::missing_field`;
   `toml_edit::ser::map::MapValueSerializer::serialize_none`.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | RQ-1 `HeaderValue`/`reqwest` deferred-error semantics; RQ-1 RFC 6265 / 7230 / 9110 / 6265bis cookie-name charset; RQ-2 dyn-compatibility + edition-2024 + rename provenance; RQ-4 serde `missing_field` / `flatten` / toml `None`; RQ-5 thiserror 1→2 deltas. (2 additional `high`-effort attempts terminated with `TypeError: terminated` and were retried at `medium`.) |
| Perplexity perplexity_reason | 0 | not needed — synthesis was done against primary sources |
| Perplexity perplexity_search | 2 | locating the chrono CHANGELOG / release-notes pages for the introduction-version questions |
| Perplexity perplexity_ask | 2 | Rust version for the dyn-compatibility rename (**used**); chrono introduction versions (**answer rejected as unreliable** — conflicted with primary sources) |
| Context7 | 0 | not used — the vendored crate sources at the exact locked versions are strictly more authoritative than any docs mirror for these questions |
| Tavily tavily_extract | 1 | cross-validating the thiserror 2.0.0 release-notes page against WebFetch |
| Tavily tavily_search / _research / _crawl / _map | 0 | not needed |
| WebFetch | 5 | Rust Reference dyn-compatibility section (verbatim); RFC 9110 §5.6.2 `token`/`tchar` ABNF (verbatim); thiserror 2.0.0 release notes (verbatim bullets); chrono `CHANGELOG.md` (**inconclusive** — covers only ≤ 0.4.19); chrono v0.4.35 release notes (**inconclusive** — no `FromStr`/relaxed mention) |
| WebSearch | 1 | disproving the hypothesised "RFC 3729 / RFC 3782" attribution for the object-safety rename |
| **Local source reads (primary evidence)** | 20 Read/Grep | `http-1.4.0/src/header/value.rs` + `src/error.rs`; `reqwest-0.12.28/src/async_impl/request.rs` + `src/error.rs`; `chrono-0.4.44/src/datetime/mod.rs`, `src/naive/datetime/mod.rs`, `src/format/parse.rs`, `src/format/scan.rs`, `src/offset/mod.rs`; `serde-1.0.228/src/private/de.rs`; `serde_derive-1.0.228/src/de.rs`; `toml-0.8.23/Cargo.toml`; `toml_edit-0.22.27/src/ser/{map,value,array,mod}.rs`; `thiserror-{1.0.69,2.0.18}/Cargo.toml`; `/Users/jmagady/Dev/prism/Cargo.lock` |
| Training data | 0 areas load-bearing | Every claim in this artifact is traceable to a quoted source or a named source file. No version number was taken from model knowledge — all were read from `Cargo.lock` or the vendored `Cargo.toml`s. |

**Total MCP tool calls:** 10 (5 `perplexity_research`, 2 `perplexity_search`, 2 `perplexity_ask`,
1 `tavily_extract`) + 6 non-MCP web calls (5 WebFetch, 1 WebSearch) + 20 local source reads.

**Training data reliance:** **low.** The decisive evidence for RQ-1 (b/c), RQ-3, RQ-4, and the RQ-5
MSRV correction is direct source reads at the exact locked versions. RQ-1(a)'s ABNF and RQ-2's
dyn-compatibility conditions are verbatim quotations from the RFC and the Rust Reference. Two
`perplexity_ask` answers were rejected after contradicting primary sources, and the two residual
INCONCLUSIVE items are explicitly scoped and shown to be decision-irrelevant.
