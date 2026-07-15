use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value, STORED, STRING, TEXT};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument};

pub struct TantivyIndex {
    index: Index,
    writer: Arc<Mutex<IndexWriter>>,
    reader: IndexReader,
    schema: Schema,
}

impl TantivyIndex {
    pub fn new(index_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(index_dir).map_err(|e| e.to_string())?;

        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("path", STORED | STRING);
        schema_builder.add_text_field("body", STORED | TEXT);
        let schema = schema_builder.build();

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(index_dir).map_err(|e| e.to_string())?,
            schema.clone(),
        )
        .map_err(|e| e.to_string())?;

        let writer = index.writer(50_000_000).map_err(|e| e.to_string())?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            index,
            writer: Arc::new(Mutex::new(writer)),
            reader,
            schema,
        })
    }

    pub fn index_document(&self, relative_path: &str, body: &str) -> Result<(), String> {
        let mut writer = self.writer.lock().unwrap();

        let path_field = self.schema.get_field("path").unwrap();
        let term = tantivy::Term::from_field_text(path_field, relative_path);
        writer.delete_term(term);

        let mut doc = TantivyDocument::default();
        doc.add_text(path_field, relative_path.to_string());
        doc.add_text(self.schema.get_field("body").unwrap(), body.to_string());
        writer.add_document(doc).map_err(|e| e.to_string())?;

        writer.commit().map_err(|e| e.to_string())?;
        let _ = self.reader.reload();
        Ok(())
    }

    pub fn delete_document(&self, relative_path: &str) -> Result<(), String> {
        let mut writer = self.writer.lock().unwrap();
        let path_field = self.schema.get_field("path").unwrap();
        let term = tantivy::Term::from_field_text(path_field, relative_path);
        writer.delete_term(term);
        writer.commit().map_err(|e| e.to_string())?;
        let _ = self.reader.reload();
        Ok(())
    }

    pub fn search(&self, query_str: &str) -> Result<Vec<(String, usize, String)>, String> {
        let _ = self.reader.reload();
        let searcher = self.reader.searcher();
        let body_field = self.schema.get_field("body").unwrap();
        let path_field = self.schema.get_field("path").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![body_field]);
        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| e.to_string())?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(100))
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument =
                searcher.doc(doc_address).map_err(|e| e.to_string())?;
            let path = retrieved_doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let body = retrieved_doc
                .get_first(body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let query_words: Vec<String> = query_str
                .to_lowercase()
                .split_whitespace()
                .map(|w| w.to_string())
                .collect();

            for (idx, line) in body.lines().enumerate() {
                let line_lower = line.to_lowercase();
                if query_words.iter().any(|word| line_lower.contains(word)) {
                    results.push((path.clone(), idx + 1, line.trim().to_string()));
                }
            }
        }

        Ok(results)
    }
}
