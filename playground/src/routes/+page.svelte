<script lang="ts">
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Nav from '$lib/components/Nav.svelte';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import Output from '$lib/components/Output.svelte';
	import { Vm } from '$lib/focus-lang/focus_lang';
	import { base } from '$app/paths';
	import type { MenuItem } from '$lib/menu';

	import * as marked from 'marked';

	let sidebar: Sidebar;
	let codeEditor: CodeEditor;
	let output: Output;
	let source: string;

	document.addEventListener('keydown', (e) => {
		if (e.ctrlKey && e.key === 's') {
			e.preventDefault();
			e.stopPropagation();
			codeEditor.save();
		}
	});

	function run() {
		if (source) {
			try {
				const vm = Vm.new_with_std();
				vm.add_event_listener('log', (data: CustomEvent<string>) => {
					output.log(data.detail);
				});
				vm.execute_from_source(source);
			} catch (error) {
				console.error(error);
				output.log(error as string);
			}
		}
	}

	async function loadMenuItem(item: CustomEvent<MenuItem>) {
		if (item.detail.markdown) {
			const content = await (await fetch(`${base}/${item.detail.markdown}`)).text();
			sidebar.setContent(await marked.parse(content));
		}

		if (item.detail.code) {
			const code = await (await fetch(`${base}/${item.detail.code}`)).text();
			source = code;
		}
	}
</script>

<main class="d-flex flex-nowrap" style="height: 100vh; overflow-x: auto; overflow-y: hidden;">
	<Sidebar
		bind:this={sidebar}
		on:selected={loadMenuItem}
		menu={[
			{
				title: 'Tutorials',
				icon: 'bi bi-book',
				items: [
					{
						title: '01. Hello World',
						id: 'tutorial-01',
						markdown: 'tutorials/01-intro.md',
						code: 'tutorials/01-intro.fl'
					}
				]
			},
			{
				title: 'Examples',
				icon: 'bi bi-journals',
				items: [
					{
						title: '01. Fibonnacci',
						id: 'example-01',
						code: 'examples/fibonacci.fl'
					}
				]
			}
		]}
	/>

	<div class="d-flex flex-column w-100">
		<Nav
			on:run={run}
			on:reset={codeEditor.reset}
			on:save={codeEditor.save}
			on:load={codeEditor.load}
		/>
		<CodeEditor bind:this={codeEditor} bind:source />

		<Output bind:this={output} />
	</div>
</main>
