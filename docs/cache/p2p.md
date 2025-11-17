# Peer to Peer

Picture this: you're working in an office, laptop humming as it rebuilds a large project. Meanwhile, your colleague sitting two desks away just finished building the exact same target five minutes ago. Their machine has everything you need, cached and ready. Yet here you are, waiting for your computer to fetch those artifacts from a server halfway across the country.

This scenario plays out thousands of times a day in software teams around the world. We've built elaborate cloud caching systems, deployed regional servers, and optimized network routes, all while ignoring the fastest network connection available: the one connecting you to your teammates on the same local network.

Fabrik's peer-to-peer caching changes this. Instead of always reaching out to distant servers, your build tools can fetch artifacts directly from nearby machines. The same artifacts, delivered in milliseconds instead of dozens or hundreds of milliseconds.

> [!NOTE]
> Peer-to-peer caching is most beneficial for teams working from the same local network, such as developers sharing an office or CI runners in the same data center. If your team is fully remote with everyone on different networks, you may not see significant benefits from this feature.

## Why This Matters

Traditional build caching follows a simple pattern: your machine talks to a server. When you need a cached artifact, you ask the server for it. The server lives in a data center somewhere, maybe close or maybe far, and responds as quickly as the internet allows.

This works well for distributed teams. Someone in San Francisco caches their build, and someone in Berlin can reuse it hours later. The cloud erases geography, at least in principle.

But what about when geography doesn't need erasing? What about the developer sitting in the same room, on the same WiFi network, who built the same artifact moments ago? Why should that request travel to the cloud and back when the answer is right there, meters away?

The peer-to-peer approach recognizes that software teams often cluster physically, even in our remote-first world. You might have an office where half the team works. You might be a developer with both a MacBook and a Linux desktop at home. You might have a dozen CI runners all provisioned in the same data center, rebuilding similar code in parallel.

In all these cases, there's a better first question than "Does the cloud have this?" That question is: "Does anyone nearby have this?"

## How It Works

When peer-to-peer is enabled, Fabrik adds a new layer to your cache hierarchy. Think of it as Layer 0.5, closer than your regional cache but further than your local disk.

Your local cache remains the fastest option. Nothing beats reading from your own SSD. But when you don't have something locally, instead of immediately reaching out to the regional cache, Fabrik first checks if any peers on your network might have it.

This discovery happens automatically using [mDNS](https://en.wikipedia.org/wiki/Multicast_DNS), the same technology that lets you print to nearby printers or stream to an Apple TV without configuring IP addresses. Your Fabrik instance announces its presence to the network and listens for others doing the same. No manual configuration needed. No IP addresses to remember. Just automatic discovery of nearby machines running Fabrik.

When you need an artifact, Fabrik queries all discovered peers in parallel. Whichever peer has it and responds first wins. The artifact gets transferred directly over your local network, usually in 1-5 milliseconds compared to 20-50 milliseconds from a regional cache.

If no peer has it, or if peer-to-peer is disabled, the request falls back to the regional cache exactly as before. The feature is transparent. It never breaks your build. It only makes it faster when nearby machines can help.

## Security and Privacy

Letting other machines access your build cache requires trust. Fabrik handles this through a combination of authentication and user consent.

Every team shares a secret, think of it as a password that proves you're part of the same group. This secret gets used to sign all peer-to-peer requests using [HMAC-SHA256](https://en.wikipedia.org/wiki/HMAC). If a machine doesn't know the secret, it can't access your cache. If someone tries to replay an old request, Fabrik detects it and rejects it.

But authentication alone isn't enough for peace of mind. You might trust your teammate but still want to know when they're accessing your cache. That's where consent comes in.

When a peer requests an artifact from your machine for the first time, you get a system notification. It tells you who's asking, their hostname and which machine they're using. You can approve them permanently, just for this session, or deny them entirely.

You control the notification behavior too. The default mode, "notify-once," shows a notification the first time each peer asks for access, then remembers your decision. If you want maximum control, use "notify-always" to approve every request. If you're on a completely trusted network, say just your own machines at home, you can use "always-allow" to skip notifications entirely.

What can peers access? Only build artifacts you've already created. They can't browse your filesystem, can't see your source code, can't execute anything on your machine. The peer-to-peer system deals purely in content-addressed blobs. A peer asks for a specific hash. If you have it, you send it. That's all.

## When to Use It

Peer-to-peer shines in specific scenarios. If you work alone, from home, on a single machine, it won't help you. There are no peers to share with. But if your team works from an office, even part-time, the benefits appear immediately.

Imagine a typical office day. The first developer arrives and runs the build. Their machine downloads everything from the cloud cache and stores it locally. The second developer arrives an hour later and runs the same build. Instead of hitting the cloud again, they fetch from the first developer's machine. The third developer gets it even faster, with two peers to choose from now. By the time the whole team is in, most builds pull from peers rather than the cloud.

The same pattern applies to developers who use multiple machines. You might do most of your work on a MacBook but occasionally need to test on a Linux desktop. When both machines are home, on the same network, they can share caches with each other. Build on one, instantly available on the other.

CI environments benefit too, particularly if you run your own runners. Cloud-based CI that spins up fresh containers for every build won't benefit, as there's no persistent network or peers to discover. But if you run Jenkins or GitLab runners on your own infrastructure, each runner becomes a potential cache source for the others. The first build in a batch pulls from the cloud. The rest pull from each other.

## Getting Started

Enabling peer-to-peer requires a single configuration block in your `.fabrik.toml`:

```toml
[p2p]
enabled = true
secret = "your-team-secret-here"
```

The secret should be at least 16 characters and shared across your team. Some teams use 1Password or similar tools. Others commit it to a private config repository. The important part is that everyone who should be able to share caches has the same secret.

Once configured, start your daemon as usual. Fabrik will automatically announce itself to the network and discover any other peers running with the same secret. You can check who's been discovered with `fabrik p2p list`.

When peers appear, they'll send consent requests the first time they need artifacts from you. Approve the ones you trust, deny the ones you don't. After that, the system runs on its own.

## The Philosophy

The best infrastructure is the kind you don't think about. It works when it can. It falls back when it can't. It never interrupts your workflow to ask for help.

Peer-to-peer caching embodies this principle. When peers are available and have what you need, you get faster builds. When they're not available, or don't have it, you get the same builds you always got. The feature adds speed but never adds brittleness.

This aligns with a broader truth about build caching: the most valuable cache is often the most local one. Your own disk is faster than a nearby machine. A nearby machine is faster than a regional server. A regional server is faster than the internet at large. Each layer adds latency but also adds reach.

Peer-to-peer simply fills a gap that cloud-first caching left open. It acknowledges that software development, despite being inherently digital and distributed, still clusters physically. Teams still gather in offices. Developers still own multiple machines. Infrastructure still lives in specific data centers. By making that physical proximity count for something, peer-to-peer caching brings some of the cloud's benefits back down to the local level.

---

For detailed configuration options, see the [configuration reference](/reference/config-file#p2p). For CLI commands, see the [CLI reference](/reference/cli#fabrik-p2p).
