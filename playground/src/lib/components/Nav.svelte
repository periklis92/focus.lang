<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';

	const dispatch = createEventDispatcher<{ run: void; reset: void; save: void; load: void }>();

	function keyHandler(e: KeyboardEvent) {
		if (e.key === 'F5') {
			e.preventDefault();

			dispatch('run');
		}
	}

	onMount(() => {
		document.addEventListener('keydown', keyHandler);
	});
</script>

<nav class="navbar navbar-expand-lg bg-body-tertiary" style="height: 105px;">
	<div class="container-fluid">
		<a class="navbar-brand" href="/" style="margin-left: 100px;">Focus Lang</a>

		<button
			class="navbar-toggler"
			type="button"
			data-bs-toggle="offcanvas"
			data-bs-target="#offcanvasNavbar"
			aria-controls="navbarScroll"
			aria-expanded="false"
			aria-label="Toggle navigation"
		>
			<span class="navbar-toggler-icon"></span>
		</button>

		<div
			class="offcanvas offcanvas-end"
			id="offcanvasNavbar"
			aria-labelledby="offcanvasNavbarLabel"
		>
			<div class="offcanvas-header">
				<h5 class="offcanvas-title" id="offcanvasNavbarLabel">Offcanvas</h5>
				<button type="button" class="btn-close" data-bs-dismiss="offcanvas" aria-label="Close"
				></button>
			</div>
			<div class="offcanvas-body">
				<ul class="navbar-nav justify-content-end flex-grow-1 pe-3">
					<li class="nav-item">
						<button
							class="nav-link rounded-0"
							aria-current="page"
							on:click|preventDefault={() => dispatch('save')}
						>
							<i class="bi bi-floppy2-fill" />
							Save
						</button>
					</li>
					<li class="nav-item">
						<button
							class="nav-link rounded-0"
							aria-current="page"
							on:click|preventDefault={() => dispatch('load')}
						>
							<i class="bi bi-file-earmark-arrow-up-fill" />
							Load
						</button>
					</li>
					<li class="nav-item">
						<button
							class="nav-link rounded-0"
							aria-current="page"
							on:click|preventDefault={() => dispatch('reset')}
						>
							<i class="bi bi-arrow-clockwise" />
							Reset
						</button>
					</li>
					<li class="nav-item">
						<button
							class="nav-link rounded-0"
							aria-current="page"
							on:click|preventDefault={() => dispatch('run')}
						>
							<i class="bi bi-play" />
							Run (F5)
						</button>
					</li>
				</ul>
			</div>
		</div>
	</div>
</nav>
