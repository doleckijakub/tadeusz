use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use tool::{Tool, ToolFieldSchema, ToolResult};

#[derive(Debug, Deserialize, Serialize, Clone, ToolFieldSchema)]
pub enum SearxngEngine {
    #[serde(rename = "ask")]
    Ask,
    #[serde(rename = "bing")]
    Bing,
    #[serde(rename = "brave")]
    Brave,
    #[serde(rename = "duckduckgo")]
    Duckduckgo,
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "marginalia")]
    Marginalia,
    #[serde(rename = "mojeek")]
    Mojeek,
    #[serde(rename = "qwant")]
    Qwant,
    #[serde(rename = "startpage")]
    Startpage,
    #[serde(rename = "yahoo")]
    Yahoo,
    #[serde(rename = "yep")]
    Yep,
    #[serde(rename = "arxiv")]
    Arxiv,
    #[serde(rename = "astrophysics data system")]
    AstrophysicsDataSystem,
    #[serde(rename = "core.ac.uk")]
    CoreAcUk,
    #[serde(rename = "crossref")]
    Crossref,
    #[serde(rename = "google scholar")]
    GoogleScholar,
    #[serde(rename = "openairedatasets")]
    Openairedatasets,
    #[serde(rename = "openairepublications")]
    Openairepublications,
    #[serde(rename = "openalex")]
    Openalex,
    #[serde(rename = "pdbe")]
    Pdbe,
    #[serde(rename = "pubmed")]
    Pubmed,
    #[serde(rename = "semantic scholar")]
    SemanticScholar,
    #[serde(rename = "springer nature")]
    SpringerNature,
    #[serde(rename = "ddg definitions")]
    DdgDefinitions,
    #[serde(rename = "encyclopaedia britannica")]
    EncyclopaediaBritannica,
    #[serde(rename = "etymonline")]
    Etymonline,
    #[serde(rename = "wikibooks")]
    Wikibooks,
    #[serde(rename = "wikiquote")]
    Wikiquote,
    #[serde(rename = "wikisource")]
    Wikisource,
    #[serde(rename = "wikispecies")]
    Wikispecies,
    #[serde(rename = "wikipedia")]
    Wikipedia,
    #[serde(rename = "wikiversity")]
    Wikiversity,
    #[serde(rename = "wikivoyage")]
    Wikivoyage,
    #[serde(rename = "wiktionary")]
    Wiktionary,
    #[serde(rename = "wolframalpha")]
    Wolframalpha,
    #[serde(rename = "wordnik")]
    Wordnik,
    #[serde(rename = "alpine linux packages")]
    AlpineLinuxPackages,
    #[serde(rename = "anaconda")]
    Anaconda,
    #[serde(rename = "arch linux wiki")]
    ArchLinuxWiki,
    #[serde(rename = "azure")]
    Azure,
    #[serde(rename = "bitbucket")]
    Bitbucket,
    #[serde(rename = "codeberg")]
    Codeberg,
    #[serde(rename = "crates.io")]
    CratesIo,
    #[serde(rename = "docker hub")]
    DockerHub,
    #[serde(rename = "elasticsearch")]
    Elasticsearch,
    #[serde(rename = "fdroid")]
    Fdroid,
    #[serde(rename = "free software directory")]
    FreeSoftwareDirectory,
    #[serde(rename = "gentoo")]
    Gentoo,
    #[serde(rename = "gitea.com")]
    GiteaCom,
    #[serde(rename = "github")]
    Github,
    #[serde(rename = "github code")]
    GithubCode,
    #[serde(rename = "gitlab")]
    Gitlab,
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "hoogle")]
    Hoogle,
    #[serde(rename = "huggingface")]
    Huggingface,
    #[serde(rename = "huggingface datasets")]
    HuggingfaceDatasets,
    #[serde(rename = "huggingface spaces")]
    HuggingfaceSpaces,
    #[serde(rename = "lib.rs")]
    LibRs,
    #[serde(rename = "mankier")]
    Mankier,
    #[serde(rename = "mdn")]
    Mdn,
    #[serde(rename = "metacpan")]
    Metacpan,
    #[serde(rename = "microsoft learn")]
    MicrosoftLearn,
    #[serde(rename = "minecraft wiki")]
    MinecraftWiki,
    #[serde(rename = "national vulnerability database")]
    NationalVulnerabilityDatabase,
    #[serde(rename = "nixos wiki")]
    NixosWiki,
    #[serde(rename = "npm")]
    Npm,
    #[serde(rename = "packagist")]
    Packagist,
    #[serde(rename = "pkg.go.dev")]
    PkgGoDev,
    #[serde(rename = "pub.dev")]
    PubDev,
    #[serde(rename = "pypi")]
    Pypi,
    #[serde(rename = "repology")]
    Repology,
    #[serde(rename = "rubygems")]
    Rubygems,
    #[serde(rename = "sourcehut")]
    Sourcehut,
    #[serde(rename = "stackoverflow")]
    Stackoverflow,
    #[serde(rename = "askubuntu")]
    Askubuntu,
    #[serde(rename = "superuser")]
    Superuser,
    #[serde(rename = "caddy.community")]
    CaddyCommunity,
    #[serde(rename = "discuss.python")]
    DiscussPython,
    #[serde(rename = "pi-hole.community")]
    PiHoleCommunity,
    #[serde(rename = "hackernews")]
    Hackernews,
    #[serde(rename = "lemmy comments")]
    LemmyComments,
    #[serde(rename = "lemmy communities")]
    LemmyCommunities,
    #[serde(rename = "lemmy posts")]
    LemmyPosts,
    #[serde(rename = "lemmy users")]
    LemmyUsers,
    #[serde(rename = "lobste.rs")]
    LobsteRs,
    #[serde(rename = "reddit")]
    Reddit,
    #[serde(rename = "reuters")]
    Reuters,
    #[serde(rename = "annas archive")]
    AnnasArchive,
    #[serde(rename = "goodreads")]
    Goodreads,
    #[serde(rename = "library genesis")]
    LibraryGenesis,
    #[serde(rename = "library of congress")]
    LibraryOfCongress,
    #[serde(rename = "openlibrary")]
    Openlibrary,
    #[serde(rename = "z-library")]
    ZLibrary,
}

impl fmt::Display for SearxngEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = serde_json::to_value(self).unwrap();
        write!(f, "{}", v.as_str().unwrap())
    }
}

#[derive(Default, Tool, Debug, Deserialize)]
#[tool(description = "Perform a web search and return results")]
pub struct WebSearch {
    #[description("The search query to look up")]
    pub query: String,
    #[description("Optional list of engines to restrict the search to. Omit to use all defaults.")]
    pub engines: Option<Vec<SearxngEngine>>,
}

#[derive(Deserialize)]
struct SearxngResponse {
    results: Vec<SearxngResult>,
}

#[derive(Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    content: Option<String>,
}

#[async_trait]
impl Tool for WebSearch {
    async fn execute(&self) -> ToolResult<String> {
        let base_url = std::env::var("SEARXNG_URL").expect("SEARXNG_URL not set");

        let mut params = vec![
            ("q".to_string(), self.query.clone()),
            ("format".to_string(), "json".to_string()),
        ];
        if let Some(engines) = &self.engines
            && !engines.is_empty()
        {
            let names = engines
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("engines".to_string(), names));
        }

        let response = reqwest::Client::new()
            .get(format!("{base_url}/search"))
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("SearXNG request error: {e}"))?
            .json::<SearxngResponse>()
            .await
            .map_err(|e| format!("SearXNG parse error: {e}"))?;

        if response.results.is_empty() {
            return Ok("No results found.".to_string());
        }

        let out = response
            .results
            .iter()
            .take(8)
            .enumerate()
            .map(|(i, r)| {
                let content = r.content.as_deref().unwrap_or("").trim();
                if content.is_empty() {
                    format!("{}. {}\n   {}", i + 1, r.title, r.url)
                } else {
                    format!("{}. {}\n   {}\n   {}", i + 1, r.title, content, r.url)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(out)
    }
}
