import { BENCHER_VERSION, BENCHER_WORDMARK } from "../../util/ext";
import AuthOverride from "../auth/AuthOverride";
import BENCHER_NAVBAR_ID from "./id";

const AuthNavbar = () => {
	return (
		<AuthOverride elementId={BENCHER_NAVBAR_ID}>
			<nav
				id={BENCHER_NAVBAR_ID}
				class="navbar"
				role="navigation"
				aria-label="main navigation"
			>
				<div class="navbar-brand">
					<a class="navbar-item" title="Console Home" href="/console">
						<img src={BENCHER_WORDMARK} width="150" alt="🐰 Bencher" />
					</a>
				</div>

				<div class="navbar-menu is-active">
					<div class="navbar-start">
						<a class="navbar-item" href="/docs/">
							Docs
						</a>
						<a class="navbar-item" href="/perf">
							Public Projects
						</a>
					</div>

					<div class="navbar-end">
						<div class="navbar-item">
							<div class="navbar-item">BETA v{BENCHER_VERSION}</div>
							<div class="navbar-item" />
							<div class="navbar-item">
								<a class="button is-outlined" href="/help/">
									<span class="icon has-text-primary">
										<i class="fas fa-life-ring" aria-hidden="true" />
									</span>
									<span>Help</span>
								</a>
							</div>
							<div class="navbar-item" />
							<a class="button is-outlined" href="/console">
								<span class="icon has-text-primary">
									<i class="fas fa-angle-left" aria-hidden="true" />
								</span>
								<span>Back to Console</span>
							</a>
							<div class="navbar-item"></div>
						</div>
					</div>
				</div>
			</nav>
		</AuthOverride>
	);
};

export default AuthNavbar;
