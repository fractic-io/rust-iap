Rust utilities for validating in-app purchases (IAP) made through the Apple App Store or Google Play Store, and parsing server-to-server change notifications (for refunds, subscription cancellations, etc.).

This util is specifically written to handle purchase events from Flutter apps (with Flutter's `in_app_purchase` package), but could probably be made to work for other front-ends.

> NOTE:
>
> The responses from the Apple API are returned in JWS (JSON Web Signature) format, which optionally allows the receiver to verify the authenticity and source of the message by verifying the signature against Apple's public key. That requires keeping the most up-to-date version of Apple's public key, keeping it cached, verifying the cryptographic signature, etc., which seems overkill for this library. We are already requesting data specifically from apple.com, and receiving the data over HTTPS, so the additional cryptography logic seems unnecessary. It may be something to add in the future. On the other hand, Google's API doesn't even provide a signature to verify.

This code is provided as-is. For the time being, attention will not be given to backwards compatibility or clear documentation. It is open-sourced mainly for the chance that snippets may be useful to others looking to do similar tasks. Eventually, this may become a real library productionized and documented for external use.
