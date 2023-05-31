import { Link, wsBatchLink } from '@rspc/client/v2';

const serverOrigin = import.meta.env.VITE_SDSERVER_ORIGIN || 'localhost:8080';

const noopLink = ((o) => ({
	exec() {},
	abort() {}
})) satisfies Link;

globalThis.isDev = import.meta.env.DEV;
globalThis.rspcLinks =
	import.meta.env.VITE_SD_DEMO_MODE === 'true'
		? [noopLink]
		: [
				// TODO
				// loggerLink({
				// 	enabled: () => getDebugState().rspcLogger
				// }),
				wsBatchLink({
					url: `ws://${serverOrigin}/rspc/ws`
				})
		  ];
