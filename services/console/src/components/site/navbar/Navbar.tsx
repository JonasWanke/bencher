import { createSignal } from "solid-js";
import {
	BENCHER_GITHUB_URL,
	BENCHER_LOGO_URL,
	BENCHER_VERSION,
} from "../../../util/ext";
import BENCHER_NAVBAR_ID from "./id";

const Navbar = () => {
	const [burger, setBurger] = createSignal(false);

	return (
		<nav
			id={BENCHER_NAVBAR_ID}
			class="navbar"
			role="navigation"
			aria-label="main navigation"
		>
			<div class="navbar-brand">
				<a
					class="navbar-item"
					title="Bencher - Continuous Benchmarking"
					href="/"
				>
					<img src={BENCHER_LOGO_URL} width="150" alt="🐰 Bencher" />
				</a>

				<button
					class={`navbar-burger ${burger() && "is-active"}`}
					aria-label="menu"
					aria-expanded="false"
					onClick={(_e) => setBurger(!burger())}
				>
					<span aria-hidden="true" />
					<span aria-hidden="true" />
					<span aria-hidden="true" />
				</button>
			</div>

			<div class={`navbar-menu ${burger() && "is-active"}`}>
				<div class="navbar-start">
					<a class="navbar-item" href="/docs">
						Docs
					</a>

					<a class="navbar-item" href="/perf">
						Projects
					</a>
					<a class="navbar-item" href="/pricing">
						Pricing
					</a>
					<a
						class="navbar-item"
						href={BENCHER_GITHUB_URL}
						target="_blank"
						rel="noreferrer"
					>
						GitHub
					</a>
				</div>

				<div class="navbar-end">
					<div class="navbar-item">
						<div class="navbar-item">BETA v{BENCHER_VERSION}</div>
						<div class="navbar-item" />
						<div class="navbar-item">
							<a class="button is-outlined" href="/help">
								<span class="icon has-text-primary">
									<i class="fas fa-life-ring" aria-hidden="true" />
								</span>
								<span>Help</span>
							</a>
						</div>
						<div class="navbar-item" />
						<div class="buttons">
							<a class="button is-light" href="/auth/login">
								Log in
							</a>
							<a class="button is-primary" href="/auth/signup">
								<strong>Sign up</strong>
							</a>
						</div>
					</div>
				</div>
			</div>
		</nav>
	);
};

export default Navbar;
