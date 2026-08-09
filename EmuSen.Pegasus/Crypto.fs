namespace EmuSen.Pegasus

open System
open System.Security.Cryptography
open System.Text

/// Join codes and the pre-shared-key envelope. Threat model in EmuSen_Pegasus.md §10.5.
module Crypto =

    [<Literal>]
    let NonceBytes = 12

    [<Literal>]
    let TagBytes = 16

    [<Literal>]
    let KeyBytes = 32

    [<Literal>]
    let private Iterations = 210_000

    /// Fixed salt: both peers must derive the same key from the code alone, with
    /// no round trip to agree on a random one. Consequences in Pegasus_Sync.md §5.
    let private salt = Encoding.UTF8.GetBytes "pegasus/v1/join"

    /// Words chosen to be unambiguous when read aloud over a phone.
    let private words =
        [| "amber"; "anchor"; "banjo"; "beacon"; "cactus"; "candle"; "cobalt"; "comet"
           "dagger"; "domino"; "ember"; "falcon"; "garnet"; "harbor"; "indigo"; "jigsaw"
           "kettle"; "lantern"; "marble"; "nutmeg"; "orbit"; "pepper"; "quartz"; "ribbon"
           "saffron"; "tulip"; "umbra"; "velvet"; "walnut"; "yonder"; "zenith"; "zodiac" |]

    /// A code like "7-lantern-quartz". Entropy is stated in Pegasus_Sync.md §5.
    let newJoinCode () =
        let pick () = words[RandomNumberGenerator.GetInt32 words.Length]
        $"{RandomNumberGenerator.GetInt32(1, 10)}-{pick ()}-{pick ()}"

    let deriveKey (joinCode: string) =
        let normalised = joinCode.Trim().ToLowerInvariant()
        Rfc2898DeriveBytes.Pbkdf2(Encoding.UTF8.GetBytes normalised, salt, Iterations, HashAlgorithmName.SHA256, KeyBytes)

    /// nonce (12) || ciphertext || tag (16). A fresh random nonce per frame, so
    /// no counter has to survive a reconnect.
    let seal (key: byte[]) (plaintext: byte[]) =
        let nonce = RandomNumberGenerator.GetBytes NonceBytes
        let cipher = Array.zeroCreate<byte> plaintext.Length
        let tag = Array.zeroCreate<byte> TagBytes
        use aes = new AesGcm(key, TagBytes)
        aes.Encrypt(nonce, plaintext, cipher, tag)
        Array.concat [ nonce; cipher; tag ]

    let openSealed (key: byte[]) (sealedBytes: byte[]) =
        if sealedBytes.Length < NonceBytes + TagBytes then
            raise (ProtocolError "sealed frame is too short to contain a nonce and tag")

        let cipherLen = sealedBytes.Length - NonceBytes - TagBytes
        let nonce = sealedBytes[0 .. NonceBytes - 1]
        let cipher = sealedBytes[NonceBytes .. NonceBytes + cipherLen - 1]
        let tag = sealedBytes[NonceBytes + cipherLen ..]
        let plain = Array.zeroCreate<byte> cipherLen
        use aes = new AesGcm(key, TagBytes)

        try
            aes.Decrypt(nonce, cipher, tag, plain)
        with :? AuthenticationTagMismatchException ->
            raise (ProtocolError "frame failed authentication: wrong join code, or the stream was tampered with")

        plain

    /// Proves both sides derived the same key without putting it on the wire.
    let respondToChallenge (key: byte[]) (challenge: byte[]) =
        use mac = new HMACSHA256(key)
        mac.ComputeHash challenge

    let newChallenge () = RandomNumberGenerator.GetBytes 32

    let verifyChallenge (key: byte[]) (challenge: byte[]) (response: byte[]) =
        CryptographicOperations.FixedTimeEquals(respondToChallenge key challenge, response)
