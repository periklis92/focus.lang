import type { PageLoad } from './$types';
import init from '../lib/focus-lang/focus_lang'

export const load = (async () => {
    await init();

    return {};
}) satisfies PageLoad;