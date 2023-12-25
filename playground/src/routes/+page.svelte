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

	document.addEventListener('keydown', (e) => {
		if (e.ctrlKey && e.key === 's') {
			e.preventDefault();
			e.stopPropagation();
			codeEditor.save();
		}
	});

	function run() {
		let source = codeEditor.getSource();
		if (source.length > 0) {
			try {
				const vm = Vm.new_with_std();
				vm.add_event_listener('log', (data: CustomEvent<string>) => {
					output.log(data.detail);
				});
				let index = vm.load_from_source('main', codeEditor.getSource());
				vm.execute_module(index, 'main');
			} catch (error) {
				console.error(error);
				output.logError(error as string);
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
			codeEditor.setSource(code);
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
					},
					{
						title: '02. Variables',
						id: 'tutorial-02',
						markdown: 'tutorials/02-variables.md',
						code: 'tutorials/02-variables.fl'
					},
					{
						title: '03. Functions',
						id: 'tutorial-03',
						markdown: 'tutorials/03-functions.md',
						code: 'tutorials/03-functions.fl'
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
		<CodeEditor bind:this={codeEditor} />

		<Output bind:this={output} />
	</div>
</main>
