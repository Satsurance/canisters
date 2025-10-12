export const ICP_NETWORKS = {
    LOCAL: 'local',
    MAINNET: 'ic',
    TESTNET: 'testnet'
};

export const ICP_MAINNET_BLOCK_EXPLORER = 'https://dashboard.internetcomputer.org';

export const ICP_CONFIG = {
    [ICP_NETWORKS.LOCAL]: {
        name: 'ICP Local',
        host: 'http://127.0.0.1:4943',
        canisters: {
            backend: import.meta.env.VITE_CANISTER_ID_ICP_CANISTER_BACKEND,
            ledger: import.meta.env.VITE_CANISTER_ID_ICRC1_LEDGER,
        }
    },
    [ICP_NETWORKS.MAINNET]: {
        name: 'ICP Mainnet',
        host: 'https://icp0.io',
        canisters: {
            backend: import.meta.env.VITE_CANISTER_ID_ICP_CANISTER_BACKEND,
            ledger: import.meta.env.VITE_CANISTER_ID_ICRC1_LEDGER,
        }
    }
};

export const getCurrentNetwork = () => {
    const isLocal = process.env.NODE_ENV === 'development';
    if (isLocal) {
        return ICP_NETWORKS.LOCAL;
    } else {
        return ICP_NETWORKS.MAINNET;
    }
};

export const getCanisterIds = (network = null) => {
    const currentNetwork = network || getCurrentNetwork();
    return ICP_CONFIG[currentNetwork]?.canisters || ICP_CONFIG[ICP_NETWORKS.LOCAL].canisters;
};
