import type { Market, OrderBookLevel, OrderBookSnapshot, PricePoint, PublicTrade } from "@/types";

const listedNgxEquitySymbols = [
  "ABBEYBANK", "ABCTRANS", "ACADEMY", "ACCESSCORP", "AFRINSURE", "AFRIPRUD", "AFROMEDIA",
  "AIICO", "AIRTELAFRI", "ALEX", "ARADEL", "AUSTINLAZ", "AVACAP", "AVAIF", "BAPLC",
  "BERGER", "BETAGLAS", "BUACEMENT", "BUAFOODS", "CADBURY", "CAP", "CAVERTON", "CHAMPION",
  "CHAMS", "CHELLARAM", "CILEASING", "CMFC", "CNIF", "CONHALLPLC", "CONOIL", "CORNERST",
  "CUSTODIAN", "CUTIX", "CWG", "DAARCOMM", "DANGCEM", "DANGSUGAR", "EKOCORP", "ELLAHLAKES",
  "ENAMELWA", "ETERNA", "ETI", "ETRANZACT", "EUNISELL", "FCMB", "FIDELITYBK", "FIDSON",
  "FIRSTHOLDCO", "FTGINSURE", "FTNCOCOA", "GEREGU", "GOLDBREW", "GTCO", "GUINEAINS",
  "GUINNESS", "HBMNG", "HMCALL", "HONYFLOUR", "IKEJAHOTEL", "IMG", "INFINITY", "INTBREW",
  "INTENEGINS", "JAIZBANK", "JAPAULGOLD", "JBERGER", "JOHNHOLT", "JULI", "LASACO", "LEARNAFRCA",
  "LEGENDINT", "LINKASSURE", "LIVESTOCK", "LIVINGTRUST", "MANSARD", "MAYBAKER", "MBENEFIT",
  "MCNICHOLS", "MECURE", "MEYER", "MOFIREIF", "MORISON", "MTNN", "MULTITREX", "MULTIVERSE",
  "NAHCO", "NASCON", "NB", "NCR", "NEIMETH", "NEM", "NESTLE", "NGXGROUP", "NIDF", "NNFM",
  "NPFMCRFBK", "NREIT", "NSLTECH", "OANDO", "OKOMUOIL", "OMATEK", "PHARMDEKO", "PREMPAINTS",
  "PRESCO", "PRESTIGE", "PZ", "REDSTAREX", "REGALINS", "RONCHESS", "ROYALEX", "RTBRISCOE",
  "SCOA", "SEPLAT", "SFSREIT", "SKYAVN", "SOVRENINS", "STANBIC", "STERLINGNG", "SUNUASSUR",
  "TANTALIZER", "THOMASWY", "TIP", "TOTAL", "TRANSCOHOT", "TRANSCORP", "TRANSEXPR", "TRANSPOWER",
  "TRIPPLEG", "UACN", "UBA", "UCAP", "UHOMREIT", "UNILEVER", "UNIONDICON", "UNITYBNK",
  "UNIVINSURE", "UPDC", "UPDCREIT", "UPL", "VERITASKAP", "VFDGROUP", "VITAFOAM", "WAPIC",
  "WEMABANK", "ZENITHBANK", "ZICHIS",
] as const;

const simulatedNgxMarkets: Market[] = listedNgxEquitySymbols.map((ticker, index) => {
  const seed = index + 1;
  const lastPrice = 10 + ((seed * 7919) % 70000) / 10;
  const change = seededNoise(seed) * lastPrice * 0.04;
  const spread = lastPrice * 0.002;

  return {
    marketId: `mkt_ngx_${ticker.toLowerCase()}`,
    symbol: `${ticker}/NGN`,
    assetName: "NGX-listed equity · simulated",
    baseAsset: ticker,
    quoteAsset: "NGN",
    lastPrice: lastPrice.toFixed(2),
    priceChange24h: change.toFixed(2),
    priceChangePercent24h: ((change / lastPrice) * 100).toFixed(2),
    bid: (lastPrice - spread / 2).toFixed(2),
    ask: (lastPrice + spread / 2).toFixed(2),
    high24h: (lastPrice + Math.abs(change)).toFixed(2),
    low24h: Math.max(lastPrice - Math.abs(change), 0.01).toFixed(2),
    volume24h: String(15000 + ((seed * 18379) % 9850000)),
    status: "OPEN",
  };
});

const baseMockMarkets: Market[] = [
  {
    marketId: "mkt_rtf_usd",
    symbol: "RTF/NGN",
    assetName: "Rabovel Treasury Fund",
    baseAsset: "RTF",
    quoteAsset: "NGN",
    lastPrice: "100.41",
    priceChange24h: "0.03",
    priceChangePercent24h: "0.03",
    bid: "100.40",
    ask: "100.43",
    high24h: "100.44",
    low24h: "100.38",
    volume24h: "3820450",
    status: "OPEN",
  },
  {
    marketId: "mkt_gif_usd",
    symbol: "GIF/NGN",
    assetName: "Global Infrastructure Fund",
    baseAsset: "GIF",
    quoteAsset: "NGN",
    lastPrice: "105.20",
    priceChange24h: "1.15",
    priceChangePercent24h: "1.11",
    bid: "105.10",
    ask: "105.35",
    high24h: "105.60",
    low24h: "103.90",
    volume24h: "1245800",
    status: "OPEN",
  },
  {
    marketId: "mkt_pref_usd",
    symbol: "PREF/NGN",
    assetName: "Prime Real Estate Fund",
    baseAsset: "PREF",
    quoteAsset: "NGN",
    lastPrice: "97.60",
    priceChange24h: "-0.85",
    priceChangePercent24h: "-0.86",
    bid: "97.45",
    ask: "97.70",
    high24h: "98.60",
    low24h: "97.20",
    volume24h: "982300",
    status: "OPEN",
  },
  {
    marketId: "mkt_pco_usd",
    symbol: "PCO/NGN",
    assetName: "Private Credit Opportunities",
    baseAsset: "PCO",
    quoteAsset: "NGN",
    lastPrice: "101.75",
    priceChange24h: "0.20",
    priceChangePercent24h: "0.20",
    bid: "101.60",
    ask: "101.90",
    high24h: "101.95",
    low24h: "101.40",
    volume24h: "445600",
    status: "OPEN",
  },
  {
    marketId: "mkt_crf_usd",
    symbol: "CRF/NGN",
    assetName: "Commodity Reserve Fund",
    baseAsset: "CRF",
    quoteAsset: "NGN",
    lastPrice: "2415.80",
    priceChange24h: "-12.40",
    priceChangePercent24h: "-0.51",
    bid: "2414.90",
    ask: "2417.10",
    high24h: "2431.00",
    low24h: "2408.50",
    volume24h: "6120000",
    status: "OPEN",
  },
  {
    marketId: "mkt_pge_usd",
    symbol: "PGE/NGN",
    assetName: "Pre-IPO Growth Equity Fund",
    baseAsset: "PGE",
    quoteAsset: "NGN",
    lastPrice: "0",
    status: "PRE_OPEN",
  },
  {
    marketId: "mkt_dangcem_ngn",
    symbol: "DANGCEM/NGN",
    assetName: "Dangote Cement Plc",
    baseAsset: "DANGCEM",
    quoteAsset: "NGN",
    lastPrice: "612.50",
    priceChange24h: "8.20",
    priceChangePercent24h: "1.36",
    bid: "611.00",
    ask: "613.00",
    high24h: "618.00",
    low24h: "603.50",
    volume24h: "1854200",
    status: "OPEN",
  },
  {
    marketId: "mkt_gtco_ngn",
    symbol: "GTCO/NGN",
    assetName: "Guaranty Trust Holding Company Plc",
    baseAsset: "GTCO",
    quoteAsset: "NGN",
    lastPrice: "68.40",
    priceChange24h: "0.75",
    priceChangePercent24h: "1.11",
    bid: "68.20",
    ask: "68.50",
    high24h: "69.10",
    low24h: "67.30",
    volume24h: "4218000",
    status: "OPEN",
  },
  {
    marketId: "mkt_zenith_ngn",
    symbol: "ZENITHBANK/NGN",
    assetName: "Zenith Bank Plc",
    baseAsset: "ZENITHBANK",
    quoteAsset: "NGN",
    lastPrice: "52.10",
    priceChange24h: "-0.40",
    priceChangePercent24h: "-0.76",
    bid: "52.00",
    ask: "52.20",
    high24h: "53.00",
    low24h: "51.80",
    volume24h: "3876000",
    status: "OPEN",
  },
  {
    marketId: "mkt_mtnn_ngn",
    symbol: "MTNN/NGN",
    assetName: "MTN Nigeria Communications Plc",
    baseAsset: "MTNN",
    quoteAsset: "NGN",
    lastPrice: "318.00",
    priceChange24h: "4.50",
    priceChangePercent24h: "1.44",
    bid: "317.00",
    ask: "318.50",
    high24h: "322.00",
    low24h: "312.00",
    volume24h: "921500",
    status: "OPEN",
  },
  {
    marketId: "mkt_airtelafri_ngn",
    symbol: "AIRTELAFRI/NGN",
    assetName: "Airtel Africa Plc",
    baseAsset: "AIRTELAFRI",
    quoteAsset: "NGN",
    lastPrice: "2180.00",
    priceChange24h: "-16.00",
    priceChangePercent24h: "-0.73",
    bid: "2175.00",
    ask: "2185.00",
    high24h: "2210.00",
    low24h: "2160.00",
    volume24h: "154200",
    status: "OPEN",
  },
  {
    marketId: "mkt_accesscorp_ngn",
    symbol: "ACCESSCORP/NGN",
    assetName: "Access Holdings Plc",
    baseAsset: "ACCESSCORP",
    quoteAsset: "NGN",
    lastPrice: "23.90",
    priceChange24h: "0.35",
    priceChangePercent24h: "1.49",
    bid: "23.80",
    ask: "24.00",
    high24h: "24.20",
    low24h: "23.40",
    volume24h: "5620000",
    status: "OPEN",
  },
  {
    marketId: "mkt_uba_ngn",
    symbol: "UBA/NGN",
    assetName: "United Bank for Africa Plc",
    baseAsset: "UBA",
    quoteAsset: "NGN",
    lastPrice: "36.25",
    priceChange24h: "0.10",
    priceChangePercent24h: "0.28",
    bid: "36.10",
    ask: "36.30",
    high24h: "36.80",
    low24h: "35.90",
    volume24h: "2941000",
    status: "OPEN",
  },
  {
    marketId: "mkt_buafoods_ngn",
    symbol: "BUAFOODS/NGN",
    assetName: "BUA Foods Plc",
    baseAsset: "BUAFOODS",
    quoteAsset: "NGN",
    lastPrice: "425.00",
    priceChange24h: "6.00",
    priceChangePercent24h: "1.43",
    bid: "423.00",
    ask: "426.00",
    high24h: "430.00",
    low24h: "417.00",
    volume24h: "328600",
    status: "OPEN",
  },
  {
    marketId: "mkt_seplat_ngn",
    symbol: "SEPLAT/NGN",
    assetName: "Seplat Energy Plc",
    baseAsset: "SEPLAT",
    quoteAsset: "NGN",
    lastPrice: "5120.00",
    priceChange24h: "-35.00",
    priceChangePercent24h: "-0.68",
    bid: "5100.00",
    ask: "5130.00",
    high24h: "5190.00",
    low24h: "5080.00",
    volume24h: "48600",
    status: "OPEN",
  },
  {
    marketId: "mkt_fbnH_ngn",
    symbol: "FBNH/NGN",
    assetName: "FBN Holdings Plc",
    baseAsset: "FBNH",
    quoteAsset: "NGN",
    lastPrice: "32.80",
    priceChange24h: "0.55",
    priceChangePercent24h: "1.71",
    bid: "32.60",
    ask: "32.90",
    high24h: "33.10",
    low24h: "32.10",
    volume24h: "1834000",
    status: "OPEN",
  },
];

export const mockNgxMarkets: Market[] = simulatedNgxMarkets;

export const mockMarkets: Market[] = [
  ...baseMockMarkets.filter((market) => !market.marketId.endsWith("_ngn")),
  ...mockNgxMarkets,
];

function seededNoise(seed: number): number {
  return (((seed * 9301 + 49297) % 233280) / 233280) - 0.5;
}

function buildOrderBookSide(
  basePrice: number,
  direction: 1 | -1,
  levels = 12,
): OrderBookLevel[] {
  let cumulative = 0;
  return Array.from({ length: levels }, (_, i) => {
    const price = basePrice + direction * (i + 1) * basePrice * 0.0006;
    const size = 500 + Math.abs(seededNoise(i * direction)) * 4000;
    cumulative += size;
    return {
      price: price.toFixed(2),
      size: size.toFixed(2),
      total: cumulative.toFixed(2),
    };
  });
}

export function getMockOrderBook(marketId: string): OrderBookSnapshot {
  const market = mockMarkets.find((m) => m.marketId === marketId) ?? mockMarkets[0];
  const base = Number(market.lastPrice) || 100;
  return {
    marketId: market.marketId,
    bids: buildOrderBookSide(base, -1),
    asks: buildOrderBookSide(base, 1).reverse(),
    updatedAt: new Date().toISOString(),
  };
}

export function getMockPriceHistory(marketId: string, points = 120): PricePoint[] {
  const market = mockMarkets.find((m) => m.marketId === marketId) ?? mockMarkets[0];
  const base = Number(market.lastPrice) || 100;
  let last = base * 0.96;

  return Array.from({ length: points }, (_, i) => {
    const timestamp = new Date();
    timestamp.setHours(timestamp.getHours() - (points - i));
    const change = seededNoise(i) * base * 0.01;
    const open = last;
    const close = Math.max(open + change, base * 0.5);
    const high = Math.max(open, close) + Math.abs(seededNoise(i + 1)) * base * 0.003;
    const low = Math.min(open, close) - Math.abs(seededNoise(i + 2)) * base * 0.003;
    last = close;
    return {
      timestamp: timestamp.toISOString(),
      open: open.toFixed(2),
      high: high.toFixed(2),
      low: low.toFixed(2),
      close: close.toFixed(2),
      volume: (1000 + Math.abs(seededNoise(i + 3)) * 20000).toFixed(0),
    };
  });
}

export function getMockRecentTrades(marketId: string, count = 25): PublicTrade[] {
  const market = mockMarkets.find((m) => m.marketId === marketId) ?? mockMarkets[0];
  const base = Number(market.lastPrice) || 100;

  return Array.from({ length: count }, (_, i) => {
    const timestamp = new Date();
    timestamp.setSeconds(timestamp.getSeconds() - i * 47);
    const price = base + seededNoise(i) * base * 0.002;
    return {
      tradeId: `ptrade_${marketId}_${i}`,
      marketId,
      price: price.toFixed(2),
      quantity: (10 + Math.abs(seededNoise(i + 5)) * 500).toFixed(2),
      side: i % 2 === 0 ? "BUY" : "SELL",
      executedAt: timestamp.toISOString(),
    };
  });
}
