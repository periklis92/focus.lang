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
			data-bs-toggle="collapse"
			data-bs-target="#navbarScroll"
			aria-controls="navbarScroll"
			aria-expanded="false"
			aria-label="Toggle navigation"
		>
			<span class="navbar-toggler-icon"></span>
		</button>

		<div class="collapse navbar-collapse" id="navbarScroll">
			<ul class="nav me-auto my-2 my-lg-0">
				<li class="nav-item fs-4">
					<button
						class="nav-link rounded-0"
						aria-current="page"
						on:click|preventDefault={() => dispatch('save')}
					>
						<i class="bi bi-floppy2-fill" />
						Save
					</button>
				</li>
				<li class="nav-item fs-4">
					<button
						class="nav-link rounded-0"
						aria-current="page"
						on:click|preventDefault={() => dispatch('load')}
					>
						<i class="bi bi-file-earmark-arrow-up-fill" />
						Load
					</button>
				</li>
				<li class="nav-item fs-4">
					<button
						class="nav-link rounded-0"
						aria-current="page"
						on:click|preventDefault={() => dispatch('reset')}
					>
						<i class="bi bi-arrow-clockwise" />
						Reset
					</button>
				</li>
				<li class="nav-item fs-4">
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
</nav>
