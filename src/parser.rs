use nom::bytes::complete::{tag, take_until, take_while1};
use nom::combinator::{all_consuming, map, opt};
use nom::error::ParseError;
use nom::multi::{fold_many_m_n, many0, many_till};
use nom::sequence::{delimited, pair, preceded, terminated};

use crate::ast::*;
use crate::Span;

mod nom_ext;
use self::nom_ext::*;

#[cfg(test)]
mod tests;

type PResult<'a, T, E> = nom::IResult<Span<'a>, T, E>;

pub fn parse_doc<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Document<'a>, E> {
    let (i, header) = opt(parse_doc_header)(i)?;

    let f = terminated(parse_blocks, wsnl);
    let mut f = all_consuming(f);
    let (i, contents) = f(i)?;

    let doc = Document { header, content: contents };
    Ok((i, doc))
}

pub fn parse_doc_header<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, DocumentHeader<'a>, E> {
    let (i, title) = preceded(tag("= "), terminated(take_until("\n"), tag("\n")))(i)?;
    // parse author
    // parse version
    let (i, attributes) = many0(parse_doc_attribute)(i)?;
    let h = DocumentHeader { title, author: None, version: None, attributes };
    Ok((i, h))
}

pub fn parse_doc_attribute<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, DocAttribute<'a>, E> {
    // FIXME: attribute unsetting
    // FIXME: attribute continuation was a thing right?
    map(
        terminated(
            pair(
                delimited(tag(":"), take_while1(|c| c != '\n' && c != ':'), tag(":")),
                opt(preceded(ws1, take_until("\n"))),
            ),
            ws_with_nl,
        ),
        |(id, value)| DocAttribute { id, unset: false, value: value.into_iter().collect() },
    )(i)
}

pub fn parse_blocks<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Blocks<'a>, E> {
    // many0(parse_block)(i)
    todo!()
}

pub fn parse_section_title<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, SectionTitle<'a>, E> {
    let parse_level = map(fold_many_m_n(1, 6, tag("="), 0, |acc, _| acc + 1), |level| level - 1);
    let mut parse_level = terminated(parse_level, ws);
    let (i, level) = parse_level(i)?;

    let (i, content) = take_line(i)?;

    Ok((i, SectionTitle { level, content }))
}

pub fn parse_paragraph<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Vec<Span<'a>>, E> {
    let (i1, (tags, _)) = many_till(take_line, tag("\n"))(i)?;
    Ok((i1, tags))
}
