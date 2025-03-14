import * as Plot from "@observablehq/plot";
import * as d3 from "d3";
import {
	type Accessor,
	type Resource,
	createEffect,
	createSignal,
} from "solid-js";
import { PerfRange } from "../../../../../config/types";
import {
	AlertStatus,
	BoundaryLimit,
	type Boundary,
	type JsonPerf,
	type JsonPerfAlert,
} from "../../../../../types/bencher";
import { addTooltips } from "./tooltip";

// Source: https://twemoji.twitter.com
// License: https://creativecommons.org/licenses/by/4.0
const WARNING_URL =
	"https://s3.amazonaws.com/public.bencher.dev/perf/warning.png";
const SIREN_URL = "https://s3.amazonaws.com/public.bencher.dev/perf/siren.png";

export interface Props {
	isConsole: boolean;
	perfData: Resource<JsonPerf>;
	range: Accessor<PerfRange>;
	lower_value: Accessor<boolean>;
	upper_value: Accessor<boolean>;
	lower_boundary: Accessor<boolean>;
	upper_boundary: Accessor<boolean>;
	perfActive: boolean[];
	width: Accessor<number>;
}

const value_end_position_key = (limit: BoundaryLimit) => {
	switch (limit) {
		case BoundaryLimit.Lower:
			return "lower_value";
		case BoundaryLimit.Upper:
			return "upper_value";
	}
};

const boundary_position_key = (limit: BoundaryLimit) => {
	switch (limit) {
		case BoundaryLimit.Lower:
			return "lower_limit";
		case BoundaryLimit.Upper:
			return "upper_limit";
	}
};

const position_label = (limit: BoundaryLimit) => {
	switch (limit) {
		case BoundaryLimit.Lower:
			return "Lower";
		case BoundaryLimit.Upper:
			return "Upper";
	}
};

const get_units = (json_perf: JsonPerf) => {
	const units = json_perf?.results?.[0]?.measure?.units;
	if (units) {
		return units;
	} else {
		return "units";
	}
};

const get_x_axis = (range: PerfRange): [string, string] => {
	switch (range) {
		case PerfRange.DATE_TIME:
			return ["date_time", "Report Date and Time"];
		case PerfRange.VERSION:
			return ["number", "Branch Version Number"];
	}
};

const is_active = (alert: JsonPerfAlert) =>
	alert?.status && alert.status == AlertStatus.Active;

// A boundary is skipped if it is defined but its limit undefined
// This indicates that the the boundary limit could not be calculated for the metric
const boundary_skipped = (
	boundary: undefined | Boundary,
	limit: undefined | number,
) => boundary && !limit;

const LinePlot = (props: Props) => {
	const [is_plotted, set_is_plotted] = createSignal(false);
	const [y_label_area_size, set_y_label_area_size] = createSignal(512);

	createEffect(() => {
		if (is_plotted()) {
			let y_axis = document.querySelector(
				"svg [aria-label='y-axis tick label']",
			);
			if (!y_axis) {
				return;
			}
			const width = y_axis.getBoundingClientRect().width;
			set_y_label_area_size(width * 1.12);
		}
	});

	const plotted = () => {
		const json_perf = props.perfData();
		// console.log(json_perf);

		if (
			typeof json_perf !== "object" ||
			json_perf === undefined ||
			json_perf === null ||
			!Array.isArray(json_perf.results)
		) {
			return;
		}

		const units = get_units(json_perf);
		const [x_axis, x_axis_label] = get_x_axis(props.range());

		const plot_arrays = [];
		const warn_arrays = [];
		const alert_arrays = [];
		let metrics_found = false;
		const colors = d3.schemeTableau10;
		const project_slug = json_perf.project.slug;
		// biome-ignore lint/complexity/noForEach: <explanation>
		json_perf.results.forEach((result, index) => {
			const perf_metrics = result.metrics;
			if (!(Array.isArray(perf_metrics) && props.perfActive[index])) {
				return;
			}

			const line_data = [];
			const lower_alert_data = [];
			const upper_alert_data = [];
			const boundary_data = [];
			const skipped_lower_data = [];
			const skipped_upper_data = [];
			// biome-ignore lint/complexity/noForEach: <explanation>
			perf_metrics.forEach((perf_metric) => {
				const datum = {
					report: perf_metric.report,
					value: perf_metric.metric?.value,
					lower_value: perf_metric.metric?.lower_value,
					upper_value: perf_metric.metric?.upper_value,
					date_time: new Date(perf_metric.start_time),
					number: perf_metric.version?.number,
					hash: perf_metric.version?.hash,
					iteration: perf_metric.iteration,
					lower_limit: perf_metric.boundary?.lower_limit,
					upper_limit: perf_metric.boundary?.upper_limit,
				};
				line_data.push(datum);

				const limit_datum = {
					date_time: datum.date_time,
					number: datum.number,
					hash: datum.hash,
					iteration: datum.iteration,
					lower_limit: datum.lower_limit,
					upper_limit: datum.upper_limit,
				};
				if (perf_metric.alert && is_active(perf_metric.alert)) {
					switch (perf_metric.alert?.limit) {
						case BoundaryLimit.Lower:
							lower_alert_data.push({
								...limit_datum,
								alert: perf_metric.alert,
							});
							break;
						case BoundaryLimit.Upper:
							upper_alert_data.push({
								...limit_datum,
								alert: perf_metric.alert,
							});
							break;
					}
				} else {
					boundary_data.push(limit_datum);
				}

				if (
					boundary_skipped(
						perf_metric.threshold?.statistic?.lower_boundary,
						perf_metric.boundary?.lower_limit,
					)
				) {
					skipped_lower_data.push({
						date_time: datum.date_time,
						number: datum.number,
						y: perf_metric.metric?.value * 0.9,
					});
				}
				if (
					boundary_skipped(
						perf_metric.threshold?.statistic?.upper_boundary,
						perf_metric.boundary?.upper_limit,
					)
				) {
					skipped_upper_data.push({
						date_time: datum.date_time,
						number: datum.number,
						y: perf_metric.metric?.value * 1.1,
					});
				}

				metrics_found = true;
			});

			const color = colors[index % 10] ?? "7f7f7f";
			// Line
			plot_arrays.push(
				Plot.line(line_data, {
					x: x_axis,
					y: "value",
					stroke: color,
				}),
			);
			// Dots
			plot_arrays.push(
				Plot.dot(line_data, {
					x: x_axis,
					y: "value",
					stroke: color,
					fill: color,
					title: (datum) => to_title(`${datum.value}`, datum, ""),
				}),
			);

			// Lower Value
			if (props.lower_value()) {
				plot_arrays.push(
					Plot.line(
						line_data,
						value_end_line(x_axis, BoundaryLimit.Lower, color),
					),
				);
				plot_arrays.push(
					Plot.dot(
						line_data,
						value_end_dot(x_axis, BoundaryLimit.Lower, color),
					),
				);
			}

			// Upper Value
			if (props.upper_value()) {
				plot_arrays.push(
					Plot.line(
						line_data,
						value_end_line(x_axis, BoundaryLimit.Upper, color),
					),
				);
				plot_arrays.push(
					Plot.dot(
						line_data,
						value_end_dot(x_axis, BoundaryLimit.Upper, color),
					),
				);
			}

			// Lower Boundary
			if (props.lower_boundary()) {
				plot_arrays.push(
					Plot.line(
						line_data,
						boundary_line(x_axis, BoundaryLimit.Lower, color),
					),
				);
				plot_arrays.push(
					Plot.dot(
						boundary_data,
						boundary_dot(x_axis, BoundaryLimit.Lower, color),
					),
				);
				warn_arrays.push(Plot.image(skipped_lower_data, warning_image(x_axis)));
			}
			alert_arrays.push(
				Plot.image(
					lower_alert_data,
					alert_image(
						x_axis,
						BoundaryLimit.Lower,
						project_slug,
						props.isConsole,
					),
				),
			);

			// Upper Boundary
			if (props.upper_boundary()) {
				plot_arrays.push(
					Plot.line(
						line_data,
						boundary_line(x_axis, BoundaryLimit.Upper, color),
					),
				);
				plot_arrays.push(
					Plot.dot(
						boundary_data,
						boundary_dot(x_axis, BoundaryLimit.Upper, color),
					),
				);
				warn_arrays.push(Plot.image(skipped_upper_data, warning_image(x_axis)));
			}
			alert_arrays.push(
				Plot.image(
					upper_alert_data,
					alert_image(
						x_axis,
						BoundaryLimit.Upper,
						project_slug,
						props.isConsole,
					),
				),
			);
		});
		// This allows the alert images to appear on top of the plot lines.
		plot_arrays.push(...warn_arrays, ...alert_arrays);

		if (metrics_found) {
			return (
				<>
					<div>
						{addTooltips(
							Plot.plot({
								x: {
									grid: true,
									label: `${x_axis_label} ➡`,
									labelOffset: 36,
								},
								y: {
									grid: true,
									label: `↑ ${units}`,
								},
								marks: plot_arrays,
								width: props.width(),
								nice: true,
								// https://github.com/observablehq/plot/blob/main/README.md#layout-options
								// For simplicity’s sake and for consistent layout across plots, margins are not automatically sized to make room for tick labels; instead, shorten your tick labels or increase the margins as needed.
								marginLeft: y_label_area_size(),
							}),
							{
								stroke: "#ed6704",
								opacity: 0.5,
								"stroke-width": "3px",
								fill: "gray",
							},
						)}
					</div>
					<>{set_is_plotted(true)}</>
				</>
			);
		} else {
			return (
				<section class="section">
					<div class="container">
						<div class="content">
							<div>
								<h3 class="title is-3">No data found</h3>
								<h4 class="subtitle is-4">{new Date(Date.now()).toString()}</h4>
							</div>
						</div>
					</div>
				</section>
			);
		}
	};

	return <>{plotted()}</>;
};

const to_title = (prefix, datum, suffix) =>
	`${prefix}\n${datum.date_time?.toLocaleString(undefined, {
		weekday: "short",
		year: "numeric",
		month: "short",
		day: "2-digit",
		hour: "2-digit",
		hour12: false,
		minute: "2-digit",
		second: "2-digit",
	})}\nVersion Number: ${datum.number}${
		datum.hash ? `\nVersion Hash: ${datum.hash}` : ""
	}\nIteration: ${datum.iteration}${suffix}`;

const value_end_line = (
	x_axis: string,
	limit: BoundaryLimit,
	color: string,
) => {
	return {
		x: x_axis,
		y: value_end_position_key(limit),
		stroke: color,
		strokeWidth: 2,
		strokeOpacity: 0.9,
		strokeDasharray: [3],
	};
};

const value_end_dot = (x_axis: string, limit: BoundaryLimit, color: string) => {
	return {
		x: x_axis,
		y: value_end_position_key(limit),
		stroke: color,
		strokeWidth: 2,
		strokeOpacity: 0.9,
		fill: color,
		fillOpacity: 0.9,
		title: (datum) => value_end_title(limit, datum, ""),
		// TODO enable this when there is an endpoint for getting a historical threshold statistic
		// That is, the statistic displayed needs to be historical, not current.
		// Just like with the Alerts.
		// href: (datum) =>
		// 	!is_active(datum.alert) &&
		// 	`/console/projects/${project_slug}/thresholds/${datum.threshold}`,
		// target: "_blank",
	};
};

const boundary_line = (x_axis: string, limit: BoundaryLimit, color) => {
	return {
		x: x_axis,
		y: boundary_position_key(limit),
		stroke: color,
		strokeWidth: 4,
		strokeOpacity: 0.666,
		strokeDasharray: [8],
	};
};

const boundary_dot = (x_axis: string, limit: BoundaryLimit, color: string) => {
	return {
		x: x_axis,
		y: boundary_position_key(limit),
		stroke: color,
		strokeWidth: 4,
		strokeOpacity: 0.666,
		fill: color,
		fillOpacity: 0.666,
		title: (datum) => limit_title(limit, datum, ""),
		// TODO enable this when there is an endpoint for getting a historical threshold statistic
		// That is, the statistic displayed needs to be historical, not current.
		// Just like with the Alerts.
		// href: (datum) =>
		// 	!is_active(datum.alert) &&
		// 	`/console/projects/${project_slug}/thresholds/${datum.threshold}`,
		// target: "_blank",
	};
};

const warning_image = (x_axis: string) => {
	return {
		x: x_axis,
		y: "y",
		src: WARNING_URL,
		width: 18,
		title: (_datum) =>
			"Boundary Limit was not calculated.\nThis can happen for a couple of reasons:\n- There is not enough data yet (n < 2) (Most Common)\n- All the metric values are the same (variance == 0)",
	};
};

const alert_image = (
	x_axis: string,
	limit: BoundaryLimit,
	project_slug: string,
	isConsole: boolean,
) => {
	return {
		x: x_axis,
		y: boundary_position_key(limit),
		src: SIREN_URL,
		width: 18,
		title: (datum) => limit_title(limit, datum, "\nClick to view Alert"),
		href: (datum) =>
			isConsole
				? `/console/projects/${project_slug}/alerts/${datum.alert?.uuid}`
				: `/perf/${project_slug}/alerts/${datum.alert?.uuid}`,
		target: "_blank",
	};
};

const value_end_title = (limit: BoundaryLimit, datum, suffix) =>
	to_title(
		`${position_label(limit)} Value: ${datum[value_end_position_key(limit)]}`,
		datum,
		suffix,
	);

const limit_title = (limit: BoundaryLimit, datum, suffix) =>
	to_title(
		`${position_label(limit)} Limit: ${datum[boundary_position_key(limit)]}`,
		datum,
		suffix,
	);

export default LinePlot;
