/** @type {import('next').NextConfig} */
const nextConfig = {
    webpack: (config, { isServer }) => {
		const experiments = config.experiments || {}
		config.experiments = { ...experiments, asyncWebAssembly: true, layers: true }
		config.module.rules.push({
			test: /\.wasm$/,
			type: "webassembly/async",
		})
		return config
	},
	reactStrictMode: true,
	staticPageGenerationTimeout: 100,
};

export default nextConfig;
