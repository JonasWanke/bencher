import type { Params } from "astro";
import { Accessor, For, Resource, createEffect, createMemo } from "solid-js";
import type { JsonAuthUser } from "../../../../types/bencher";
import { fmtValues, setPageTitle } from "../../../../util/resource";
import { pathname, useNavigate } from "../../../../util/url";
import DeckHeaderButton, { DeckHeaderButtonConfig } from "./DeckHeaderButton";

export interface Props {
	apiUrl: string;
	params: Params;
	user: JsonAuthUser;
	config: DeckHeaderConfig;
	path: Accessor<string>;
	data: Resource<Record<string, any>>;
	handleRefresh: () => void;
}

export interface DeckHeaderConfig {
	key: string;
	keys?: string[][];
	path: (pathname: string) => string;
	path_to: string;
	buttons: DeckHeaderButtonConfig[];
}

const DeckHeader = (props: Props) => {
	const navigate = useNavigate();

	const title = createMemo(() =>
		fmtValues(props.data(), props.config?.key, props.config?.keys, " | "),
	);

	createEffect(() => {
		setPageTitle(title()?.toString());
	});

	return (
		<div class="columns is-centered">
			<div class="column is-narrow">
				<button
					class="button is-outlined is-fullwidth"
					title={`Back to ${props.config?.path_to}`}
					onClick={(e) => {
						e.preventDefault();
						navigate(props.config?.path(pathname()));
					}}
				>
					<span class="icon">
						<i class="fas fa-chevron-left" aria-hidden="true" />
					</span>
					<span>Back</span>
				</button>
			</div>
			<div class="column">
				<div class="content has-text-centered">
					<h3 class="title is-3" style="word-break: break-word;">
						{title()}
					</h3>
				</div>
			</div>

			<div class="column is-narrow">
				<nav class="level">
					<div class="level-right">
						<For each={props.config?.buttons}>
							{(button) => (
								<div class="level-item">
									<DeckHeaderButton
										apiUrl={props.apiUrl}
										params={props.params}
										user={props.user}
										button={button}
										path={props.path}
										data={props.data}
										title={title}
										handleRefresh={props.handleRefresh}
									/>
								</div>
							)}
						</For>
					</div>
				</nav>
			</div>
		</div>
	);
};

export default DeckHeader;
