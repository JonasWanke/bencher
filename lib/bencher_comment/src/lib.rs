use std::{collections::BTreeMap, time::Duration};

use bencher_json::{
    project::perf::{LOWER_BOUNDARY, UPPER_BOUNDARY},
    AlertUuid, BenchmarkName, BenchmarkUuid, BranchUuid, DateTime, JsonBoundary, JsonPerfQuery,
    JsonReport, MeasureUuid, ResourceName, Slug, TestbedUuid,
};
use url::Url;

pub struct ReportComment {
    endpoint_url: Url,
    project_slug: Slug,
    json_report: JsonReport,
    benchmark_urls: BenchmarkUrls,
    alert_urls: AlertUrls,
}

impl ReportComment {
    pub fn new(endpoint_url: Url, json_report: JsonReport) -> Self {
        Self {
            alert_urls: AlertUrls::new(&endpoint_url, &json_report),
            benchmark_urls: BenchmarkUrls::new(endpoint_url.clone(), &json_report),
            project_slug: json_report.project.slug.clone(),
            json_report,
            endpoint_url,
        }
    }

    pub fn text(&self) -> String {
        let mut comment = String::new();

        comment.push_str("View results:");
        for (benchmark, measures) in &self.benchmark_urls.0 {
            for (measure, MeasureData { console_url, .. }) in measures {
                comment.push_str(&format!(
                    "\n- {benchmark_name} ({measure_name}): {console_url}",
                    benchmark_name = benchmark.name,
                    measure_name = measure.name
                ));
            }
        }

        if self.json_report.alerts.is_empty() {
            return comment;
        }

        comment.push_str("\nView alerts:");
        for ((benchmark, measure), AlertData { console_url, .. }) in &self.alert_urls.0 {
            comment.push_str(&format!(
                "\n- {benchmark_name} ({measure_name}): {console_url}",
                benchmark_name = benchmark.name,
                measure_name = measure.name,
            ));
        }

        comment
    }

    pub fn html(&self, with_metrics: bool, require_threshold: bool, id: Option<&str>) -> String {
        let mut html = String::new();
        let html_mut = &mut html;
        let public_links = self.json_report.project.visibility.is_public();
        self.html_header(html_mut);
        self.html_report_table(html_mut, public_links);
        self.html_benchmarks_table(html_mut, with_metrics, require_threshold, public_links);
        self.html_footer(html_mut);
        // DO NOT MOVE: The Bencher tag must be the last thing in the HTML for updates to work
        self.html_bencher_tag(html_mut, id);
        html
    }

    fn html_header(&self, html: &mut String) {
        html.push_str(&format!(
            r#"<h1><a href="{endpoint_url}"><img src="https://bencher.dev/favicon.svg" width="32" height="32" alt="🐰" /></a>Bencher</h1>"#,
            endpoint_url = self.endpoint_url,
        ));
    }

    fn html_report_table(&self, html: &mut String, public_links: bool) {
        html.push_str("<table>");
        for (row, name, path) in [
            (
                "Report",
                self.json_report
                    .start_time
                    .into_inner()
                    .format("%a, %B %e, %Y at %X %Z")
                    .to_string(),
                (!public_links).then_some(format!(
                    "/console/projects/{}/reports/{}",
                    self.project_slug, self.json_report.uuid
                )),
            ),
            (
                "Project",
                self.json_report.project.name.to_string(),
                if public_links {
                    Some(format!("/perf/{}", self.project_slug))
                } else {
                    Some(format!("/console/projects/{}", self.project_slug))
                },
            ),
            (
                "Branch",
                self.json_report.branch.name.to_string(),
                (!public_links).then_some(format!(
                    "/console/projects/{}/branches/{}",
                    self.project_slug, self.json_report.branch.slug
                )),
            ),
            (
                "Testbed",
                self.json_report.testbed.name.to_string(),
                (!public_links).then_some(format!(
                    "/console/projects/{}/testbeds/{}",
                    self.project_slug, self.json_report.testbed.slug
                )),
            ),
        ] {
            if let Some(path) = path {
                let url = self.endpoint_url.clone();
                let url = url.join(&path).unwrap_or(url);
                html.push_str(&format!(
                    r#"<tr><td>{row}</td><td><a href="{url}">{name}</a></td></tr>"#
                ));
            } else {
                html.push_str(&format!(r#"<tr><td>{row}</td><td>{name}</td></tr>"#));
            }
        }
        html.push_str("</table>");
    }

    fn html_benchmarks_table(
        &self,
        html: &mut String,
        with_metrics: bool,
        require_threshold: bool,
        public_links: bool,
    ) {
        let Some((_benchmark, measures)) = self.benchmark_urls.0.first_key_value() else {
            html.push_str("<b>No benchmarks found!</b>");
            return;
        };
        html.push_str("<table>");
        self.html_benchmarks_table_header(
            html,
            measures,
            with_metrics,
            require_threshold,
            public_links,
        );
        self.html_benchmarks_table_body(html, with_metrics, require_threshold, public_links);
        html.push_str("</table>");
    }

    fn html_benchmarks_table_header(
        &self,
        html: &mut String,
        measures: &MeasuresMap,
        with_metrics: bool,
        require_threshold: bool,
        public_links: bool,
    ) {
        html.push_str("<tr>");
        html.push_str("<th>Benchmark</th>");
        for (measure, MeasureData { boundary, .. }) in measures {
            if require_threshold && boundary.is_none() {
                continue;
            }
            let measure_name = &measure.name;
            if public_links {
                html.push_str(&format!("<th>{measure_name}</th>"));
            } else {
                let measure_path = format!(
                    "/console/projects/{}/measures/{}",
                    self.project_slug, measure.slug
                );
                let url = self.endpoint_url.clone();
                let url = url.join(&measure_path).unwrap_or(url);
                html.push_str(&format!(r#"<th><a href="{url}">{measure_name}</a></th>"#));
            }

            if with_metrics {
                let units = &measure.units;
                // If there is a boundary then we will show the percentage difference
                if boundary.is_some() {
                    html.push_str(&format!(
                        "<th>{measure_name} Results<br/>{units} | (Δ%)</th>",
                    ));
                } else {
                    html.push_str(&format!("<th>{measure_name} Results<br/>{units}</th>",));
                }

                let Some(boundary) = boundary else {
                    continue;
                };
                if boundary.lower_limit.is_some() {
                    html.push_str(&format!(
                        "<th>{measure_name} Lower Boundary<br/>{units} | (%)</th>"
                    ));
                }
                if boundary.upper_limit.is_some() {
                    html.push_str(&format!(
                        "<th>{measure_name} Upper Boundary<br/>{units} | (%)</th>"
                    ));
                }
            }
        }
        html.push_str("</tr>");
    }

    #[allow(clippy::too_many_lines)]
    fn html_benchmarks_table_body(
        &self,
        html: &mut String,
        with_metrics: bool,
        require_threshold: bool,
        public_links: bool,
    ) {
        for (benchmark, measures) in &self.benchmark_urls.0 {
            html.push_str("<tr>");
            if public_links {
                html.push_str(&format!("<td>{name}</td>", name = benchmark.name,));
            } else {
                let benchmark_path = format!(
                    "/console/projects/{}/benchmarks/{}",
                    self.project_slug, benchmark.slug
                );
                let url = self.endpoint_url.clone();
                let url = url.join(&benchmark_path).unwrap_or(url);
                html.push_str(&format!(
                    r#"<td><a href="{url}">{name}</a></td>"#,
                    name = benchmark.name,
                ));
            }
            for (
                measure,
                MeasureData {
                    public_url,
                    console_url,
                    value,
                    boundary,
                },
            ) in measures
            {
                if require_threshold && boundary.is_none() {
                    continue;
                }
                let plot_url = if public_links {
                    public_url
                } else {
                    console_url
                };
                let alert_url = self
                    .alert_urls
                    .0
                    .get(&(benchmark.clone(), measure.clone()))
                    .map(
                        |AlertData {
                             public_url,
                             console_url,
                         }| {
                            if public_links {
                                public_url
                            } else {
                                console_url
                            }
                        },
                    );
                let row = if let Some(alert_url) = alert_url {
                    format!(
                        r#"🚨 (<a href="{plot_url}">view plot</a> | <a href="{alert_url}">view alert</a>)"#,
                    )
                } else if boundary.is_some() {
                    format!(r#"✅ (<a href="{plot_url}">view plot</a>)"#)
                } else {
                    format!(r#"➖ (<a href="{plot_url}">view plot</a>)"#)
                };
                html.push_str(&format!(r#"<td>{row}</td>"#));

                if with_metrics {
                    let value = *value;
                    // If there is a boundary with a baseline then show the percentage difference
                    if let Some(Boundary {
                        baseline: Some(baseline),
                        ..
                    }) = boundary
                    {
                        let value_percent = if value.is_normal() && baseline.is_normal() {
                            ((value - baseline) / baseline) * 100.0
                        } else {
                            0.0
                        };
                        let value_plus = if value_percent > 0.0 { "+" } else { "" };

                        html.push_str(&format!(
                            "<td>{value:.3} ({value_plus}{value_percent:.2}%)</td>"
                        ));
                    } else {
                        html.push_str(&format!("<td>{value:.3}</td>"));
                    }

                    let Some(boundary) = boundary else {
                        continue;
                    };
                    if let Some(lower_limit) = boundary.lower_limit {
                        let limit_percent = if value.is_normal() && lower_limit.is_normal() {
                            (lower_limit / value) * 100.0
                        } else {
                            0.0
                        };
                        html.push_str(&format!("<td>{lower_limit:.3} ({limit_percent:.2}%)</td>"));
                    }
                    if let Some(upper_limit) = boundary.upper_limit {
                        let limit_percent = if value.is_normal() && upper_limit.is_normal() {
                            (value / upper_limit) * 100.0
                        } else {
                            0.0
                        };
                        html.push_str(&format!("<td>{upper_limit:.3} ({limit_percent:.2}%)</td>"));
                    }
                }
            }
            html.push_str("</tr>");
        }
    }

    fn html_footer(&self, html: &mut String) {
        html.push_str(&format!(r#"<br/><small><a href="https://bencher.dev">Bencher - Continuous Benchmarking</a></small>{}<br/><small><a href="https://bencher.dev/docs/">Docs</a> | <a href="https://bencher.dev/repo/">Repo</a> | <a href="https://bencher.dev/chat/">Chat</a> | <a href="https://bencher.dev/help/">Help</a></small>"#,
        if self.json_report.project.visibility.is_public() {
            let path = format!("/perf/{}", self.project_slug);
            let url = self.endpoint_url.clone();
            let url = url.join(&path).unwrap_or(url);
            format!(r#"<br/><small><a href="{url}">View Public Perf Page</a></small>"#)
        } else {
            String::new()
        }
        ));
    }

    fn html_bencher_tag(&self, html: &mut String, id: Option<&str>) {
        html.push_str(&self.bencher_tag(id));
    }

    // The Bencher tag allows us to easily check whether a comment is a Bencher report when updating
    pub fn bencher_tag(&self, id: Option<&str>) -> String {
        let id = id.map_or_else(
            || {
                format!(
                    "{branch}/{testbed}/{adapter:?}",
                    branch = self.json_report.branch.uuid,
                    testbed = self.json_report.testbed.uuid,
                    adapter = self.json_report.adapter
                )
            },
            ToString::to_string,
        );
        format!(
            r#"<div id="bencher.dev/projects/{project}/id/{id}"></div>"#,
            project = self.json_report.project.uuid,
        )
    }

    pub fn has_threshold(&self) -> bool {
        self.benchmark_urls.has_threshold()
    }

    pub fn has_alert(&self) -> bool {
        !self.json_report.alerts.is_empty()
    }
}

pub struct BenchmarkUrls(BTreeMap<Benchmark, MeasuresMap>);
pub type MeasuresMap = BTreeMap<Measure, MeasureData>;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Benchmark {
    name: BenchmarkName,
    slug: Slug,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Measure {
    name: ResourceName,
    slug: Slug,
    units: ResourceName,
}

#[derive(Clone)]
pub struct MeasureData {
    pub public_url: Url,
    pub console_url: Url,
    pub value: f64,
    pub boundary: Option<Boundary>,
}

impl BenchmarkUrls {
    pub fn new(endpoint_url: Url, json_report: &JsonReport) -> Self {
        let benchmark_url = BenchmarkUrl::new(
            endpoint_url,
            json_report.project.slug.clone(),
            json_report.branch.uuid,
            json_report.testbed.uuid,
            json_report.start_time,
            json_report.end_time,
        );

        let mut urls = BTreeMap::new();
        if let Some(iteration) = json_report.results.first() {
            for result in iteration {
                let measure = Measure {
                    name: result.measure.name.clone(),
                    slug: result.measure.slug.clone(),
                    units: result.measure.units.clone(),
                };
                for benchmark_metric in &result.benchmarks {
                    let benchmark = Benchmark {
                        name: benchmark_metric.name.clone(),
                        slug: benchmark_metric.slug.clone(),
                    };
                    let benchmark_urls = urls.entry(benchmark).or_insert_with(BTreeMap::new);
                    let boundary = benchmark_metric.boundary.map(Into::into);

                    let data = MeasureData {
                        public_url: benchmark_url.to_public_url(
                            benchmark_metric.uuid,
                            result.measure.uuid,
                            boundary,
                        ),
                        console_url: benchmark_url.to_console_url(
                            benchmark_metric.uuid,
                            result.measure.uuid,
                            boundary,
                        ),
                        value: benchmark_metric.metric.value.into(),
                        boundary,
                    };
                    benchmark_urls.insert(measure.clone(), data);
                }
            }
        }

        Self(urls)
    }

    fn has_threshold(&self) -> bool {
        self.0.values().any(Self::benchmark_has_threshold)
    }

    fn benchmark_has_threshold(measures: &MeasuresMap) -> bool {
        measures
            .values()
            .any(|MeasureData { boundary, .. }| boundary.is_some())
    }
}

struct BenchmarkUrl {
    endpoint: Url,
    project_slug: Slug,
    branch: BranchUuid,
    testbed: TestbedUuid,
    start_time: DateTime,
    end_time: DateTime,
}

// 30 days
const DEFAULT_REPORT_HISTORY: Duration = Duration::from_secs(30 * 24 * 60 * 60);

impl BenchmarkUrl {
    fn new(
        endpoint: Url,
        project_slug: Slug,
        branch: BranchUuid,
        testbed: TestbedUuid,
        start_time: DateTime,
        end_time: DateTime,
    ) -> Self {
        Self {
            endpoint,
            project_slug,
            branch,
            testbed,
            start_time,
            end_time,
        }
    }

    fn to_public_url(
        &self,
        benchmark: BenchmarkUuid,
        measure: MeasureUuid,
        boundary: Option<Boundary>,
    ) -> Url {
        self.to_url(benchmark, measure, boundary, true)
    }

    fn to_console_url(
        &self,
        benchmark: BenchmarkUuid,
        measure: MeasureUuid,
        boundary: Option<Boundary>,
    ) -> Url {
        self.to_url(benchmark, measure, boundary, false)
    }

    fn to_url(
        &self,
        benchmark: BenchmarkUuid,
        measure: MeasureUuid,
        boundary: Option<Boundary>,
        public_links: bool,
    ) -> Url {
        let json_perf_query = JsonPerfQuery {
            branches: vec![self.branch],
            testbeds: vec![self.testbed],
            benchmarks: vec![benchmark],
            measures: vec![measure],
            start_time: Some((self.start_time.into_inner() - DEFAULT_REPORT_HISTORY).into()),
            end_time: Some(self.end_time),
        };

        let mut url = self.endpoint.clone();
        let path = if public_links {
            format!("/perf/{}", self.project_slug)
        } else {
            format!("/console/projects/{}/perf", self.project_slug)
        };
        url.set_path(&path);
        url.set_query(Some(
            &json_perf_query
                .to_query_string(&boundary.map(Boundary::to_query_string).unwrap_or_default())
                .unwrap_or_default(),
        ));

        url
    }
}

#[derive(Clone, Copy)]
pub struct Boundary {
    baseline: Option<f64>,
    lower_limit: Option<f64>,
    upper_limit: Option<f64>,
}

impl From<JsonBoundary> for Boundary {
    fn from(json_boundary: JsonBoundary) -> Self {
        Self {
            baseline: json_boundary.baseline.map(Into::into),
            lower_limit: json_boundary.lower_limit.map(Into::into),
            upper_limit: json_boundary.upper_limit.map(Into::into),
        }
    }
}

impl Boundary {
    fn to_query_string(self) -> Vec<(&'static str, Option<String>)> {
        let mut query_string = Vec::new();
        if self.lower_limit.is_some() {
            query_string.push((LOWER_BOUNDARY, Some(true.to_string())));
        }
        if self.upper_limit.is_some() {
            query_string.push((UPPER_BOUNDARY, Some(true.to_string())));
        }
        query_string
    }

    pub fn is_empty(self) -> bool {
        self.lower_limit.is_none() && self.upper_limit.is_none()
    }
}

pub struct AlertUrls(BTreeMap<(Benchmark, Measure), AlertData>);

#[derive(Clone)]
pub struct AlertData {
    pub public_url: Url,
    pub console_url: Url,
}

impl AlertUrls {
    pub fn new(endpoint_url: &Url, json_report: &JsonReport) -> Self {
        let mut urls = BTreeMap::new();

        for alert in &json_report.alerts {
            let benchmark = Benchmark {
                name: alert.benchmark.name.clone(),
                slug: alert.benchmark.slug.clone(),
            };
            let measure = Measure {
                name: alert.threshold.measure.name.clone(),
                slug: alert.threshold.measure.slug.clone(),
                units: alert.threshold.measure.units.clone(),
            };
            let public_url =
                Self::to_public_url(endpoint_url.clone(), &json_report.project.slug, alert.uuid);
            let console_url =
                Self::to_console_url(endpoint_url.clone(), &json_report.project.slug, alert.uuid);
            let data = AlertData {
                public_url,
                console_url,
            };
            urls.insert((benchmark, measure), data);
        }

        Self(urls)
    }

    fn to_public_url(mut endpoint: Url, project_slug: &Slug, alert: AlertUuid) -> Url {
        endpoint.set_path(&format!("/perf/{project_slug}/alerts/{alert}"));
        endpoint
    }

    fn to_console_url(mut endpoint: Url, project_slug: &Slug, alert: AlertUuid) -> Url {
        endpoint.set_path(&format!("/console/projects/{project_slug}/alerts/{alert}"));
        endpoint
    }
}
