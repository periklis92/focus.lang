<script lang="ts">
	import type { Menu, MenuItem } from '$lib/menu';
	import { afterUpdate, createEventDispatcher, type EventDispatcher } from 'svelte';

	export let menu: Menu[];
	export let content: string | undefined = undefined;
	export const dispatcher = createEventDispatcher<{ selected: MenuItem }>();

	export function setContent(html: string) {
		content = html;
	}

	afterUpdate(window.Prism?.highlightAll);
</script>

<div class="position-relative bg-dark">
	<div class="collapse collapse-horizontal show h-100" id="collapseExample">
		<div class=" flex-shrink-0 p-3 m-0 h-100" style="width: 380px; overflow-y:auto;">
			<a
				href="/"
				class="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-white text-decoration-none"
			>
				<span class="fs-4">Menu</span>
			</a>
			<hr />
			{#if !content}
				<ul class="nav nav-pills flex-column mb-auto">
					{#each menu as item, i}
						<li class="nav-item mb-2">
							<a
								href="/"
								class="d-flex mb-3 mb-md-0 me-md-auto text-white text-decoration-none"
								type="button"
								data-bs-toggle="collapse"
								data-bs-target={`#_menu-item-${i}`}
								aria-expanded="false"
								aria-controls={`_menu-item-${i}`}
							>
								{#if item.icon}
									<i class={`${item.icon}`} style="margin-right: 4px;" />
								{/if}
								{item.title}
							</a>
							<div class="collapse" id={`_menu-item-${i}`}>
								<ul class="btn-toggle-nav list-unstyled fw-normal pb-1 small">
									{#each item.items as child}
										<li>
											<a
												id={child.id}
												href="/"
												class=" text-white"
												on:click|preventDefault={() => dispatcher('selected', child)}
												>{child.title}</a
											>
										</li>
									{/each}
								</ul>
							</div>
						</li>
					{/each}
				</ul>
			{:else}
				<div class="container" style="">
					<button
						type="button"
						class="btn btn-primary btn-sm sticky-top"
						aria-current="page"
						on:click|preventDefault={() => (content = undefined)}
					>
						<i class="bi bi-arrow-left" />
						Back
					</button>
					<div class="content">
						{@html content}
					</div>
				</div>
			{/if}
		</div>
	</div>
	<button
		class="btn btn-secondary position-absolute translate-end badge"
		style="width: 80px; z-index: 9999;height: 50px; left: 100%; top: 15px; margin-left:15px;"
		type="button"
		data-bs-toggle="collapse"
		data-bs-target="#collapseExample"
		aria-expanded="false"
		aria-controls="collapseExample"
	>
		<i class="bi bi-list fs-2" />
	</button>
</div>

<div class="divider bg-primary" />

<style>
	.divider {
		flex-shrink: 0;
		width: 0.15rem;
		height: 100vh;
		background-color: rgb(48, 48, 49);
		border: solid rgba(0, 0, 0, 0.15);
		border-width: 1px 0;
		box-shadow:
			inset 0 0.5em 1.5em rgba(255, 255, 255, 0.1),
			inset 0 0.125em 0.5em rgba(255, 255, 255, 0.438);
	}

	.btn-toggle-nav a {
		padding: 0.1875rem 0.5rem;
		margin-top: 0.125rem;
		margin-left: 1.25rem;
	}

	.content:not(code) {
		color: white;
	}
</style>
