# Satsurance UI

UI created for interactions with Satsurance smart-contracts.

## Configuration

### API Host Configuration

The application automatically routes API requests based on the deployment environment:

- **Local Development** (`npm run dev`): API requests use Vite's proxy configuration (proxies to `localhost:3050` by default)
- **Production Build** (`npm run build`): API requests are sent to `https://testnet.satsurance.xyz`

This is configured in `vite.config.js` and requires no additional setup.

### Environment Variables

Create a `.env` file in the frontend directory with the following variables:

```env
# Optional: Override proxy target for local development (defaults to http://localhost:3050)
# VITE_API_PROXY=http://localhost:3050

# Canister IDs for local network
VITE_CANISTER_ID_ICP_CANISTER_BACKEND_LOCAL=<canister-id>
VITE_CANISTER_ID_ICRC1_LEDGER_LOCAL=<canister-id>

# Canister IDs for mainnet
VITE_CANISTER_ID_ICP_CANISTER_BACKEND_MAINNET=<canister-id>
VITE_CANISTER_ID_ICRC1_LEDGER_MAINNET=<canister-id>
```

## Development

```bash
npm install
npm run dev
```

## Building for Production

```bash
npm run build
```

The production build will automatically configure API requests to use `testnet.satsurance.xyz`.