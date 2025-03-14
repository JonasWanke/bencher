import type { Params } from "astro";
import FieldKind from "../../components/field/kind";
import { isAllowedProjectDelete } from "../../util/auth";
import { ActionButton, Button, Card, Display, Operation, Row } from "../types";
import { addPath, createdUuidPath, parentPath, viewUuidPath } from "../util";

const paginationUrl = (
	params: undefined | Params,
	dimension: string,
	per_page: number,
	page: number,
) => {
	const searchParams = new URLSearchParams();
	searchParams.set("per_page", per_page?.toString());
	searchParams.set("page", page?.toString());
	const url = `/v0/projects/${
		params?.project
	}/${dimension}?${searchParams.toString()}`;
	return url;
};

const THRESHOLD_FIELDS = {
	branch: {
		name: "branches",
		icon: "fas fa-code-branch",
		option_key: "name",
		value_key: "uuid",
		url: (params: undefined | Params, per_page: number, page: number) =>
			paginationUrl(params, "branches", per_page, page),
	},
	testbed: {
		name: "testbeds",
		icon: "fas fa-server",
		option_key: "name",
		value_key: "uuid",
		url: (params: undefined | Params, per_page: number, page: number) =>
			paginationUrl(params, "testbeds", per_page, page),
	},
	measure: {
		name: "measures",
		icon: "fas fa-shapes",
		option_key: "name",
		value_key: "uuid",
		url: (params: undefined | Params, per_page: number, page: number) =>
			paginationUrl(params, "measures", per_page, page),
	},
};

const thresholdsConfig = {
	[Operation.LIST]: {
		operation: Operation.LIST,
		header: {
			title: "Thresholds",
			buttons: [
				{
					kind: Button.ADD,
					title: "Threshold",
					path: addPath,
				},
				{ kind: Button.REFRESH },
			],
		},
		table: {
			url: (params: Params) => `/v0/projects/${params?.project}/thresholds`,
			add: {
				prefix: (
					<div>
						<h4>🐰 Create your first threshold!</h4>
						<p>
							It's easy to create a new threshold.
							<br />
							<a href="/docs/explanation/thresholds/">Learn about thresholds</a>{" "}
							or tap below to get started.
						</p>
					</div>
				),
				path: addPath,
				text: "Add a Threshold",
			},
			row: {
				keys: [
					["branch", "name"],
					["testbed", "name"],
					["measure", "name"],
				],
				items: [
					{
						kind: Row.NESTED_TEXT,
						keys: ["branch", "name"],
					},
					{},
					{
						kind: Row.NESTED_TEXT,
						keys: ["testbed", "name"],
					},
					{
						kind: Row.NESTED_TEXT,
						keys: ["measure", "name"],
					},
				],
				button: {
					text: "View",
					path: viewUuidPath,
				},
			},
			name: "thresholds",
		},
	},
	[Operation.ADD]: {
		operation: Operation.ADD,
		header: {
			title: "Add Threshold",
			path: parentPath,
			path_to: "Thresholds",
		},
		form: {
			url: (params: Params) => `/v0/projects/${params?.project}/thresholds`,
			fields: [
				{
					kind: FieldKind.RADIO,
					label: "Branch",
					key: "branch",
					value: "",
					valid: null,
					validate: true,
					config: THRESHOLD_FIELDS.branch,
				},
				{
					kind: FieldKind.RADIO,
					label: "Testbed",
					key: "testbed",
					value: "",
					valid: null,
					validate: true,
					config: THRESHOLD_FIELDS.testbed,
				},
				{
					kind: FieldKind.RADIO,
					label: "Measure",
					key: "measure",
					value: "",
					valid: null,
					validate: true,
					config: THRESHOLD_FIELDS.measure,
				},
				{
					kind: FieldKind.STATISTIC,
					label: null,
					key: "statistic",
					value: {},
					validate: true,
					config: null,
				},
			],
			path: createdUuidPath,
		},
	},
	[Operation.VIEW]: {
		operation: Operation.VIEW,
		header: {
			keys: [
				["branch", "name"],
				["testbed", "name"],
				["measure", "name"],
			],
			path: parentPath,
			path_to: "Thresholds",
			buttons: [
				{ kind: Button.EDIT, path: (pathname: string) => `${pathname}/edit` },
				{ kind: Button.REFRESH },
			],
		},
		deck: {
			url: (params: Params) =>
				`/v0/projects/${params?.project}/thresholds/${params?.threshold}`,
			cards: [
				{
					kind: Card.FIELD,
					label: "Threshold UUID",
					key: "uuid",
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Branch",
					keys: ["branch", "name"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Testbed",
					keys: ["testbed", "name"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Measure",
					keys: ["measure", "name"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Statistical Significance Test",
					keys: ["statistic", "test"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Lower Boundary",
					keys: ["statistic", "lower_boundary"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Upper Boundary",
					keys: ["statistic", "upper_boundary"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Minimum Sample Size",
					keys: ["statistic", "min_sample_size"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Maximum Sample Size",
					keys: ["statistic", "max_sample_size"],
					display: Display.RAW,
				},
				{
					kind: Card.NESTED_FIELD,
					label: "Window Size (seconds)",
					keys: ["statistic", "window"],
					display: Display.RAW,
				},
			],
			buttons: [
				{
					kind: ActionButton.DELETE,
					subtitle:
						"⚠️ All Reports that use this Threshold must be deleted first! ⚠️",
					path: parentPath,
					is_allowed: isAllowedProjectDelete,
				},
			],
		},
	},
	[Operation.EDIT]: {
		operation: Operation.EDIT,
		header: {
			title: "Edit Threshold Statistic",
			path: parentPath,
			path_to: "Threshold",
		},
		form: {
			url: (params: Params) =>
				`/v0/projects/${params?.project}/thresholds/${params?.threshold}`,
			fields: [
				{
					kind: FieldKind.STATISTIC,
					label: null,
					key: "statistic",
					value: {},
					validate: true,
					config: null,
				},
			],
			path: parentPath,
		},
	},
};

export default thresholdsConfig;
