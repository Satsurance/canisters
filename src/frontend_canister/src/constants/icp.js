export const ICP_NETWORKS = {
    LOCAL: 'local',
    MAINNET: 'ic',
    TESTNET: 'testnet'
};

export const ICP_CONFIG = {
    [ICP_NETWORKS.LOCAL]: {
        name: 'ICP Local',
        host: 'http://127.0.0.1:4943',
        canisters: {
            backend: import.meta.env.VITE_CANISTER_ID_ICP_CANISTER_BACKEND || 'ufxgi-4p777-77774-qaadq-cai',
            ledger: import.meta.env.VITE_CANISTER_ID_ICRC1_LEDGER || 'ulvla-h7777-77774-qaacq-cai',
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
    return import.meta.env.VITE_DFX_NETWORK === 'ic' ? ICP_NETWORKS.MAINNET : ICP_NETWORKS.LOCAL;
};

export const getCanisterIds = (network = null) => {
    const currentNetwork = network || getCurrentNetwork();
    return ICP_CONFIG[currentNetwork]?.canisters || ICP_CONFIG[ICP_NETWORKS.LOCAL].canisters;
};
