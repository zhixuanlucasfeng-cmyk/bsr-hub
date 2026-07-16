import type { DemoOrder, Fulfillment, Listing, OrderState, Quote } from "./types.ts";

export const demoCatalog: Listing[] = [
  { id:"ps5-slim", ownerId:"seller-demo", title:"PS5 Slim + Two Controllers", listingType:"rental", category:"Gaming", description:"Quiet, clean console with Spider-Man 2 and HDMI cable. Great for a weekend with friends.", city:"Wellesley", state:"MA", condition:"Like new", unitPriceCents:1200, depositCents:10000, deliveryFeeCents:800, billingUnit:"thirty_minutes", fulfillment:["pickup","delivery","owner_location"], accent:"#6d5dfc", icon:"🎮", imageSrc:"/images/listings/ps5-slim.jpg", imageAlt:"White PlayStation 5 console with two controllers" },
  { id:"ps5-pro", ownerId:"seller-demo", title:"PS5 Pro Creator Setup", listingType:"rental", category:"Gaming", description:"High-performance console with headset and capture card for streams or tournaments.", city:"Newton", state:"MA", condition:"Excellent", unitPriceCents:1800, depositCents:15000, deliveryFeeCents:1200, billingUnit:"thirty_minutes", fulfillment:["pickup","delivery"], accent:"#8b5cf6", icon:"🕹️", imageSrc:"/images/listings/ps5-pro.jpg", imageAlt:"Gaming console, controller, and streaming headset setup" },
  { id:"macbook", ownerId:"tech-demo", title:"MacBook Pro M2 14-inch", listingType:"rental", category:"Computers", description:"Portable editing and development laptop with charger and protective sleeve.", city:"Needham", state:"MA", condition:"Excellent", unitPriceCents:5200, depositCents:20000, deliveryFeeCents:1000, billingUnit:"day", fulfillment:["pickup","delivery"], accent:"#0ea5e9", icon:"💻", imageSrc:"/images/listings/macbook.jpg", imageAlt:"Open MacBook Pro laptop on a clean desk" },
  { id:"gaming-pc", ownerId:"tech-demo", title:"RTX Gaming & Rendering PC", listingType:"rental", category:"Computers", description:"RTX 4070 desktop for gaming, 3D work, and short production projects.", city:"Waltham", state:"MA", condition:"Very good", unitPriceCents:6800, depositCents:25000, deliveryFeeCents:1800, billingUnit:"day", fulfillment:["delivery","owner_location"], accent:"#06b6d4", icon:"🖥️", imageSrc:"/images/listings/gaming-pc.jpg", imageAlt:"RGB desktop computer and monitor gaming setup" },
  { id:"sony-camera", ownerId:"creator-demo", title:"Sony A7 IV Camera Kit", listingType:"rental", category:"Cameras", description:"Full-frame camera, 28–70mm lens, batteries, SD card, and compact tripod.", city:"Brookline", state:"MA", condition:"Excellent", unitPriceCents:7500, depositCents:30000, deliveryFeeCents:1200, billingUnit:"day", fulfillment:["pickup","delivery"], accent:"#f97316", icon:"📷", imageSrc:"/images/listings/sony-camera.jpg", imageAlt:"Mirrorless camera with lens and creator accessories" },
  { id:"podcast-kit", ownerId:"creator-demo", title:"Two-Person Podcast Kit", listingType:"rental", category:"Cameras", description:"Two microphones, audio interface, table arms, headphones, and cables.", city:"Boston", state:"MA", condition:"Good", unitPriceCents:4200, depositCents:12000, deliveryFeeCents:1000, billingUnit:"day", fulfillment:["pickup","delivery"], accent:"#fb7185", icon:"🎙️", imageSrc:"/images/listings/podcast-kit.jpg", imageAlt:"Two microphones and headphones arranged for a podcast" },
  { id:"tool-set", ownerId:"maker-demo", title:"Cordless Home Project Tool Set", listingType:"rental", category:"Tools", description:"Drill, driver, sander, batteries, safety glasses, and organized case.", city:"Dedham", state:"MA", condition:"Good", unitPriceCents:2800, depositCents:8000, deliveryFeeCents:900, billingUnit:"day", fulfillment:["pickup","delivery"], accent:"#f59e0b", icon:"🛠️", imageSrc:"/images/listings/tool-set.jpg", imageAlt:"Cordless drill and home project tools on a workbench" },
  { id:"laser-cutter", ownerId:"maker-demo", title:"Desktop Laser Cutter Session", listingType:"workspace", category:"Maker space", description:"Supervised laser cutter and ventilation station. Materials charged separately.", city:"Cambridge", state:"MA", condition:"Professional", unitPriceCents:2200, depositCents:0, deliveryFeeCents:0, billingUnit:"thirty_minutes", fulfillment:["on_site"], accent:"#eab308", icon:"⚙️", imageSrc:"/images/listings/laser-cutter.jpg", imageAlt:"Laser cutting machine in a supervised maker workshop" },
  { id:"photo-studio", ownerId:"seller-demo", title:"Natural-Light Photo Studio", listingType:"workspace", category:"Studio", description:"500 sq ft studio with backdrops, lights, changing area, and freight elevator.", city:"Boston", state:"MA", condition:"Professional", unitPriceCents:3500, depositCents:5000, deliveryFeeCents:0, billingUnit:"thirty_minutes", fulfillment:["on_site"], accent:"#ec4899", icon:"🎬", imageSrc:"/images/listings/photo-studio.jpg", imageAlt:"Bright photography studio with lights and backdrop" },
  { id:"print-shop", ownerId:"business-demo", title:"Small-Batch Print Workshop", listingType:"workspace", category:"Printing", description:"Book presses, cutters, worktables, and help from an experienced operator.", city:"Somerville", state:"MA", condition:"Professional", unitPriceCents:3000, depositCents:0, deliveryFeeCents:0, billingUnit:"thirty_minutes", fulfillment:["on_site"], accent:"#14b8a6", icon:"🖨️", imageSrc:"/images/listings/print-shop.jpg", imageAlt:"Small print workshop with presses and worktables" },
  { id:"monitor-sale", ownerId:"tech-demo", title:"Used 27-inch 4K Monitor", listingType:"sale", category:"Second-hand", description:"Color-accurate display with stand and cables. Small cosmetic mark on rear shell.", city:"Newton", state:"MA", condition:"Very good", unitPriceCents:16500, depositCents:0, deliveryFeeCents:1500, billingUnit:"day", fulfillment:["pickup","delivery"], accent:"#3b82f6", icon:"🖥️", imageSrc:"/images/listings/monitor-sale.jpg", imageAlt:"Twenty-seven inch monitor on a desktop" },
  { id:"camera-sale", ownerId:"creator-demo", title:"Second-Hand Instant Camera", listingType:"sale", category:"Second-hand", description:"Tested instant camera with case and one unopened film pack.", city:"Wellesley", state:"MA", condition:"Good", unitPriceCents:6500, depositCents:0, deliveryFeeCents:700, billingUnit:"day", fulfillment:["pickup","delivery"], accent:"#a855f7", icon:"📸", imageSrc:"/images/listings/camera-sale.jpg", imageAlt:"Second-hand instant camera with film pack" },
];

const transitions: Partial<Record<OrderState, Record<string, OrderState>>> = {
  pending_payment: { mark_paid:"paid", cancel:"cancelled", expire:"expired" },
  paid: { confirm:"confirmed", cancel:"cancelled" },
  confirmed: { activate:"active", fulfill:"fulfilled", cancel:"cancelled" },
  active: { return:"returned" },
  returned: { complete:"completed" },
  fulfilled: { complete:"completed" },
};

export function createHubStaticDemo() {
  const orders: DemoOrder[] = [];
  let sequence = 1;

  function listing(id: string) {
    const found = demoCatalog.find((item) => item.id === id);
    if (!found) throw new Error("Listing not found");
    return found;
  }

  function quote(listingId: string, units: number, fulfillment: Fulfillment): Quote {
    const item = listing(listingId);
    if (!Number.isInteger(units) || units <= 0) throw new Error("Units must be positive");
    if (!item.fulfillment.includes(fulfillment)) throw new Error("That fulfillment option is unavailable");
    const baseCents = item.unitPriceCents * units;
    const serviceFeeCents = Math.trunc(baseCents * 600 / 10_000);
    const deliveryFeeCents = fulfillment === "delivery" ? item.deliveryFeeCents : 0;
    return { unitPriceCents:item.unitPriceCents, billableUnits:units, billingUnit:item.billingUnit, baseCents, serviceFeeCents, deliveryFeeCents, depositCents:item.depositCents, totalCents:baseCents + serviceFeeCents + deliveryFeeCents + item.depositCents, currency:"USD" };
  }

  return {
    listings: () => demoCatalog.map((item) => ({ ...item, fulfillment:[...item.fulfillment] })),
    ordersFor: (persona: string) => orders.filter((order) => order.buyerId === persona || order.sellerId === persona).map((order) => ({ ...order, quote:{...order.quote} })),
    quote,
    createOrder(listingId: string, units: number, fulfillment: Fulfillment) {
      const item = listing(listingId);
      const order: DemoOrder = { id:`hub-demo-${sequence++}`, listingId:item.id, listingTitle:item.title, buyerId:"buyer-demo", sellerId:item.ownerId, state:"pending_payment", fulfillment, quote:quote(listingId, units, fulfillment) };
      orders.push(order);
      return { ...order, quote:{...order.quote} };
    },
    act(orderId: string, action: string) {
      const order = orders.find((item) => item.id === orderId);
      if (!order) throw new Error("Order not found");
      const next = transitions[order.state]?.[action];
      if (!next) throw new Error("Action is not allowed in the current state");
      order.state = next;
      return { ...order, quote:{...order.quote} };
    },
  };
}

export function getDemoListing(id: string) {
  return demoCatalog.find((listing) => listing.id === id);
}

export const hubStaticDemo = createHubStaticDemo();
