import { Language } from "../i18n/ui";

// This document was automatically generated by OpenAI GPT-4. It might not be accurate and might contain errors. If you find any errors, please <>open an issue on GitHub</>.

export const Disclosure = (props: {
	bodyText: string;
	linkText: string;
	page: string;
	lang: Language;
}) => {
	return (
		<div class="box" style="margin-top: 4rem;">
			🤖 {props.bodyText}
			<a
				href={`https://github.com/bencherdev/bencher/issues/new?title=Issue+with+translation+to+${props.lang}&body=Issue+with+translation+to+${props.lang}+for+${props.page}&labels=documentation`}
				target="_blank"
			>
				{props.linkText}
			</a>
			.
		</div>
	);
};

export const DisclosureDe = (props: { page: string }) => {
	return (
		<Disclosure
			bodyText="Dieses Dokument wurde automatisch von OpenAI GPT-4 generiert. Es ist möglicherweise nicht korrekt und kann Fehler enthalten. Wenn Sie Fehler finden, "
			linkText="öffnen Sie bitte ein Problem auf GitHub"
			page={props.page}
			lang={Language.de}
		/>
	);
};

export const DisclosureEs = (props: { page: string }) => {
	return (
		<Disclosure
			bodyText="Este documento fue generado automáticamente por OpenAI GPT-4. Puede que no sea exacto y contenga errores. Si encuentra algún error, "
			linkText="abra un problema en GitHub"
			page={props.page}
			lang={Language.es}
		/>
	);
};

export const DisclosureFr = (props: { page: string }) => {
	return (
		<Disclosure
			bodyText="Ce document a été automatiquement généré par OpenAI GPT-4. Il peut ne pas être précis et peut contenir des erreurs. Si vous trouvez des erreurs, veuillez "
			linkText="ouvrir une issue sur GitHub"
			page={props.page}
			lang={Language.fr}
		/>
	);
};

export const DisclosureJa = (props: { page: string }) => {
	return (
		<Disclosure
			bodyText="このドキュメントは OpenAI GPT-4 によって自動的に生成されました。 正確ではない可能性があり、間違いが含まれている可能性があります。 エラーを見つけた場合は、"
			linkText="GitHub で問題を開いてください。"
			page={props.page}
			lang={Language.ja}
		/>
	);
};

export const DisclosurePt = (props: { page: string }) => {
	return (
		<Disclosure
			bodyText="Este documento foi gerado automaticamente pelo OpenAI GPT-4. Pode não ser preciso e pode conter erros. Se você encontrar algum erro, "
			linkText="abra um problema no GitHub"
			page={props.page}
			lang={Language.pt}
		/>
	);
};

export const DisclosureZh = (props: { page: string }) => {
	return (
		<Disclosure
			bodyText="该文档由 OpenAI GPT-4 自动生成。 它可能不准确并且可能包含错误。 如果您发现任何错误，请"
			linkText="在 GitHub 上提出问题"
			page={props.page}
			lang={Language.zh}
		/>
	);
};
