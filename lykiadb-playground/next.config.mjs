/** @type {import('next').NextConfig} */
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import { fileURLToPath } from 'url';
import { resolve, dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const nextConfig = {
    webpack: (config, { isServer }) => {
		const experiments = config.experiments || {}
		config.experiments = { 
			...experiments,
			asyncWebAssembly: true,
			syncWebAssembly: true,
			layers: true
		}
        config.plugins.push(new WasmPackPlugin({
            crateDirectory: resolve(__dirname, "."),
			watchDirectories: [
				resolve(__dirname, "../lykiadb-lang/src")
			],
			extraArgs: "--target web --mode normal"
        }))
		return config
	},
	reactStrictMode: true,
	staticPageGenerationTimeout: 100,
};

export default nextConfig;
