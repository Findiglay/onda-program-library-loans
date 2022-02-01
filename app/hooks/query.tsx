import * as anchor from "@project-serum/anchor";
import { AnchorWallet } from "@solana/wallet-adapter-react";
import bs58 from "bs58";
import { useQuery } from "react-query";
import * as api from "../lib/api";

export function useNFTByOwnerQuery(
  connection: anchor.web3.Connection,
  pubkey: anchor.web3.PublicKey | null
) {
  return useQuery(
    ["wallet-nfts", pubkey?.toBase58()],
    () => {
      if (pubkey) {
        return api.getNFTs(connection, pubkey);
      }
    },
    {
      enabled: Boolean(pubkey),
      refetchOnWindowFocus: false,
    }
  );
}

export type NFTResult = api.NFTResult;

export function useMetadataFileQuery(uri?: string) {
  return useQuery(
    ["metadataFile", uri],
    () => {
      if (uri) {
        return fetch(uri).then((response) => {
          return response.json().then((data) => data);
        });
      }
    },
    {
      enabled: Boolean(uri),
      refetchOnWindowFocus: false,
    }
  );
}

export function useListingsQuery(connection: anchor.web3.Connection) {
  return useQuery(
    ["listings"],
    () =>
      api.getListings(connection, [
        {
          memcmp: {
            // filter listed
            offset: 7 + 1,
            bytes: bs58.encode(
              new anchor.BN(api.ListingState.Listed).toArrayLike(Buffer)
            ),
          },
        },
      ]),
    {
      refetchOnWindowFocus: false,
    }
  );
}

export function useListingsByOwnerQuery(
  connection: anchor.web3.Connection,
  wallet?: AnchorWallet
) {
  return useQuery(
    ["listings", wallet?.publicKey.toBase58()],
    () => {
      if (wallet) {
        return api.getListings(connection, [
          {
            memcmp: {
              // filter listed
              offset: 7 + 1,
              bytes: bs58.encode(
                new anchor.BN(api.ListingState.Listed).toArrayLike(Buffer)
              ),
            },
          },
          {
            memcmp: {
              // filter borrower
              offset: 7 + 1 + 8 + 1,
              bytes: wallet?.publicKey.toBase58(),
            },
          },
        ]);
      }
    },
    {
      enabled: Boolean(wallet),
      refetchOnWindowFocus: false,
    }
  );
}

export function useLoansQuery(
  connection: anchor.web3.Connection,
  wallet?: AnchorWallet
) {
  return useQuery(
    ["loans", wallet?.publicKey.toBase58()],
    () => {
      if (wallet) {
        return api.getListings(connection, [
          {
            memcmp: {
              // filter active
              offset: 7 + 1,
              bytes: bs58.encode(
                new anchor.BN(api.ListingState.Active).toArrayLike(Buffer)
              ),
            },
          },
          {
            memcmp: {
              // filter lender
              offset: 7 + 1 + 8 + 32 + 1,
              bytes: wallet?.publicKey.toBase58(),
            },
          },
        ]);
      }
    },
    {
      enabled: Boolean(wallet?.publicKey),
      refetchOnWindowFocus: false,
    }
  );
}

export function useBorrowingsQuery(
  connection: anchor.web3.Connection,
  wallet?: AnchorWallet
) {
  return useQuery(
    ["borrowings", wallet?.publicKey.toBase58()],
    () => {
      if (wallet) {
        return api.getListings(connection, [
          {
            memcmp: {
              // filter active
              offset: 7 + 1,
              bytes: bs58.encode(
                new anchor.BN(api.ListingState.Active).toArrayLike(Buffer)
              ),
            },
          },
          {
            memcmp: {
              // filter borrower
              offset: 7 + 1 + 8 + 1,
              bytes: wallet?.publicKey.toBase58(),
            },
          },
        ]);
      }
    },
    {
      enabled: Boolean(wallet?.publicKey),
      refetchOnWindowFocus: false,
    }
  );
}
