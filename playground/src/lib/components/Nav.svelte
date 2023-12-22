<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{ run: void; reset: void }>();
</script>

<nav class="navbar navbar-expand-lg bg-body-tertiary" style="height: 105px;">
	<div class="container-fluid">
		<a class="navbar-brand" href="/" style="margin-left: 100px;">Focus Lang</a>
		<ul class="nav nav-pills mb-auto text-center mr-0 ml-auto">
			<li class="nav-item fs-3">
				<button
					class="nav-link rounded-0"
					aria-current="page"
					on:click|preventDefault={() => dispatch('reset')}
				>
					<i class="bi bi-arrow-clockwise" />
					Reset
				</button>
			</li>
			<li class="nav-item fs-3">
				<button
					class="nav-link rounded-0"
					aria-current="page"
					on:click|preventDefault={() => dispatch('run')}
				>
					<i class="bi bi-play" />
					Run
				</button>
			</li>
		</ul>
	</div>
</nav>
