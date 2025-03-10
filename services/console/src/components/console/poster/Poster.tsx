import bencher_valid_init from "bencher_valid";
import { For, Show, createResource, createSignal } from "solid-js";
import Field, { type FieldConfig, type FieldValue } from "../../field/Field";
import FieldKind from "../../field/kind";
import { createStore } from "solid-js/store";
import { authUser } from "../../../util/auth";
import { pathname } from "../../../util/url";
import { validJwt } from "../../../util/valid";
import { Operation, Resource, resourceSingular } from "../../../config/types";
import { httpPost, httpPut } from "../../../util/http";
import type { Params } from "astro";
import { NotifyKind, navigateNotify, pageNotify } from "../../../util/notify";

export interface Props {
	apiUrl: string;
	params: Params;
	resource: Resource;
	operation: Operation;
	config: PosterConfig;
}

export interface PosterConfig {
	url: (params: Params) => string;
	fields: PosterFieldConfig[];
	path: (pathname: string, resp: any) => string;
	button: string;
}

export interface PosterFieldConfig {
	kind: FieldKind;
	label: string;
	key: string;
	value: FieldValue;
	valid: null | boolean;
	validate: boolean;
	config: FieldConfig;
	nullable?: null | boolean;
}

export type PosterForm = Record<string, PosterField>;

export interface PosterField {
	kind: FieldKind;
	label: string;
	value: FieldValue;
	valid: null | boolean;
	validate: boolean;
	nullable: undefined | null | boolean;
}

const initForm = (fields: PosterFieldConfig[]) => {
	let newForm: PosterForm = {};
	fields.forEach((field) => {
		if (field.key) {
			newForm[field.key] = {
				kind: field.kind,
				label: field.label,
				value: field.value,
				valid: field.valid,
				validate: field.validate,
				nullable: field.nullable,
			};
		}
	});
	return newForm;
};

const Poster = (props: Props) => {
	const [bencher_valid] = createResource(
		async () => await bencher_valid_init(),
	);
	const [form, setForm] = createStore(initForm(props.config?.fields));
	const [submitting, setSubmitting] = createSignal(false);
	const [valid, setValid] = createSignal(false);

	const isSendable = (): boolean => {
		return !submitting() && valid();
	};

	const httpOperation = async (
		path: string,
		token: string,
		data: Record<string, undefined | number | string>,
	) => {
		switch (props.operation) {
			case Operation.EDIT:
				return await httpPut(props.apiUrl, path, token, data);
			case Operation.ADD:
			default:
				return await httpPost(props.apiUrl, path, token, data);
		}
	};

	const sendForm = () => {
		if (!bencher_valid()) {
			return;
		}
		const token = authUser()?.token;
		if (!validJwt(token)) {
			return;
		}
		if (!isSendable()) {
			return;
		}

		setSubmitting(true);
		const data: Record<string, undefined | number | string> = {};
		for (const key of Object.keys(form)) {
			const value = form?.[key]?.value;
			switch (form?.[key]?.kind) {
				case FieldKind.SELECT:
					if (form?.[key]?.nullable && !value?.selected) {
						continue;
					}
					data[key] = value?.selected;
					break;
				case FieldKind.NUMBER:
					if (form?.[key]?.nullable && !value) {
						continue;
					}
					data[key] = Number(value);
					break;
				case FieldKind.STATISTIC:
					// Flatten the statistic object
					for (const [k, v] of Object.entries(value)) {
						data[k] = v;
					}
					break;
				default:
					if (form?.[key]?.nullable && !value) {
						continue;
					}
					if (typeof value === "string") {
						data[key] = value.trim();
					} else {
						data[key] = value;
					}
			}
		}

		const path = props.config?.url?.(props.params);
		httpOperation(path, token, data)
			.then((resp) => {
				setSubmitting(false);
				navigateNotify(
					NotifyKind.OK,
					`Hare's to your success! You've posted ${resourceSingular(
						props.resource,
					)}.`,
					props.config?.path?.(pathname(), resp.data),
					null,
					null,
				);
			})
			.catch((error) => {
				setSubmitting(false);
				console.error(error);
				pageNotify(
					NotifyKind.ERROR,
					`Lettuce romaine calm! Failed to post ${resourceSingular(
						props.resource,
					)}. Please, try again.`,
				);
			});
	};

	const handleField = (key: string, value: FieldValue, valid: boolean) => {
		if (key && form?.[key]) {
			if (form?.[key]?.nullable && !value) {
				value = null;
				valid = true;
			}

			setForm({
				...form,
				[key]: {
					...form?.[key],
					value,
					valid,
				},
			});

			setValid(isValid());
		}
	};

	const isValid = () => {
		const form_values = Object.values(form);
		for (let i = 0; i < form_values.length; i++) {
			if (form_values[i]?.validate && !form_values[i]?.valid) {
				return false;
			}
		}
		return true;
	};

	return (
		<div class="columns">
			<div class="column">
				<form class="box">
					<For each={props.config?.fields}>
						{(field, _i) => (
							<Field
								apiUrl={props.apiUrl}
								kind={field?.kind}
								label={form?.[field?.key]?.label}
								fieldKey={field?.key}
								value={form?.[field?.key]?.value}
								valid={form?.[field?.key]?.valid}
								config={field?.config}
								params={props.params}
								handleField={handleField}
							/>
						)}
					</For>
					<br />
					<div class="field">
						<p class="control">
							<button
								class="button is-primary is-fullwidth"
								type="submit"
								disabled={!isSendable()}
								onClick={(e) => {
									e.preventDefault();
									sendForm();
								}}
							>
								<Show when={props.config?.button} fallback={"Save"}>
									{props.config?.button}
								</Show>
							</button>
						</p>
					</div>
				</form>
			</div>
		</div>
	);
};

export default Poster;
