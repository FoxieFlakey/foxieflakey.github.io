use std::{cmp, sync::Arc};

use codemap::File;
// Bridge between CodeMap's Arc<File> and miette so
// SourceCode can be used
#[derive(Clone, Debug)]
pub struct MietteFile(pub Arc<File>);

impl miette::SourceCode for MietteFile {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError>
    {
        let span = self.0.span.subspan(span.offset().try_into().unwrap(), (span.offset() + span.len()).try_into().unwrap());
        
        let line_start = self.0.find_line(span.low()).saturating_sub(context_lines_before);
        let line_end = cmp::min(self.0.find_line(span.high()) + context_lines_after, self.0.num_lines());
        
        let line_start_offset = self.0.line_span(line_start).low() - self.0.span.low();
        let line_end_offset = self.0.line_span(line_end).high() - self.0.span.low();
        
        let full_span = self.0.span.subspan(line_start_offset, line_end_offset);
        
        Ok(Box::new(miette::MietteSpanContents::new_named(
            self.0.name().into(),
            self.0.source_slice(full_span).as_bytes(),
            miette::SourceSpan::new(
                miette::SourceOffset::from(usize::try_from(line_start_offset).unwrap()),
                (line_end_offset - line_start_offset).try_into().unwrap()
            ),
            line_start,
            0,
            line_end - line_start
        )))
    }
}

pub fn into_miette_span(file: &codemap::File, span: &codemap::Span) -> miette::SourceSpan {
    let start = usize::try_from(span.low() - file.span.low()).unwrap();
    let end = usize::try_from(span.high() - file.span.low()).unwrap();
    miette::SourceSpan::new(
        miette::SourceOffset::from(start),
        end - start
    )
}

