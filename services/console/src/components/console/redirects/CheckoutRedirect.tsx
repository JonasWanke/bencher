import bencher_valid_init from "bencher_valid";
import { authUser } from "../../../util/auth";
import { NotifyKind, navigateNotify } from "../../../util/notify";
import { useNavigate, useSearchParams } from "../../../util/url";
import { PLAN_PARAM } from "../../auth/auth";
import { createEffect, createResource } from "solid-js";
import {
	validJwt,
	validOptionUuid,
	validPlanLevel,
	validU32,
} from "../../../util/valid";
import { httpPost } from "../../../util/http";
import type { JsonNewPlan } from "../../../types/bencher";

export interface Props {
	apiUrl: string;
	organization: undefined | string;
}

const CheckoutRedirect = (props: Props) => {
	const [bencher_valid] = createResource(
		async () => await bencher_valid_init(),
	);
	const user = authUser();
	const [searchParams, _setSearchParams] = useSearchParams();
	const navigate = useNavigate();

	createEffect(() => {
		if (!bencher_valid()) {
			return;
		}
		const token = user?.token;
		if (!validJwt(token)) {
			return;
		}

		const checkout = searchParams["checkout"];
		const level = searchParams["level"];
		const entitlementsString = searchParams["entitlements"];
		const entitlements = parseInt(entitlementsString);
		const self_hosted = searchParams["self_hosted"];
		const billing = `/console/organizations/${props.organization}/billing`;
		if (
			!checkout ||
			!validPlanLevel(level) ||
			(entitlementsString && !validU32(entitlements)) ||
			(self_hosted && !validOptionUuid(self_hosted))
		) {
			return;
		}

		const newPlan: JsonNewPlan = {
			checkout,
			level,
			entitlements,
			self_hosted,
		};

		httpPost(
			props.apiUrl,
			`/v0/organizations/${props.organization}/plan`,
			token,
			newPlan,
		)
			.then((resp) => {
				navigateNotify(
					NotifyKind.OK,
					"Somebunny loves us! Successful plan enrollment.",
					billing,
					null,
					null,
					true,
				);
			})
			.catch((error) => {
				console.error(error);
				navigateNotify(
					NotifyKind.ERROR,
					"Lettuce romaine calm! Failed to enroll. Please, try again.",
					billing,
					null,
					[[PLAN_PARAM, level]],
					true,
				);
			});
	});

	return <></>;
};

export default CheckoutRedirect;
