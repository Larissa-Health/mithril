searchState.loadedDescShard("mithril_signer", 0, "Mithril Signer crate documentation\nClient configuration\nCritical error means the runtime will exit and the …\nDefault configuration with all the default values for …\nParse file error\nAdapter to convert EpochSettingsMessage to …\nStarting state\nKeepState error means the runtime will keep its state and …\nCould not associate my node with a stake.\nCould not find the stake for one of the signers.\nValue was expected from a subsystem but None was returned.\n<code>ReadyToSign</code> state. The signer is registered and ready to …\n<code>RegisteredNotAbleToSign</code> state. The signer is registered …\nThis trait is mainly intended for mocking.\nThis type represents the errors thrown from the Runner.\nRuntimeError Error kinds tied to their faith in the state …\nController methods for the Signer’s state machine.\nDifferent possible states of the state machine.\nThe state machine is responsible of the execution of the …\nAdapter to create RegisterSignerMessage from Signer …\nHold the latest known epoch in order to help …\nAggregator endpoint\nIf set no error is returned in case of unparsable block …\nCreate era reader adapter from configuration settings.\nCheck if the signer can sign the current epoch.\nCardano CLI tool path\nPath of the socket used by the Cardano CLI tool to …\nThe maximum number of roll forwards during a poll of the …\nThe maximum number of roll forwards during a poll of the …\nCreate the message to be signed with the single signature.\nCreate the single signature.\nPerform a cycle of the state machine.\nDirectory to store signer data (Stakes, Protocol …\ndatabase module. This module contains the entities …\nDirectory to snapshot\nDependency injection module.\nDisable immutables digests cache.\nEnable metrics server (Prometheus endpoint on /metrics).\nIf set, the signer will prune the cardano transactions in …\nTransaction pruning toggle\nEntities module\nEra reader adapter parameters\nEra reader adapter type\nEra reader adapter type\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nFetch the beacon to sign if any.\nFetch the current time point from the Cardano node.\nFetch the current epoch settings if any.\nReturn the CardanoNetwork value from the configuration.\nCreate the SQL store directory if not exist and return the …\nReturn the current state of the state machine.\nRegister epoch information\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nEasy matching Critical errors.\nReturns <code>true</code> if the state in <code>Init</code>\nReturns <code>true</code> if the state in <code>ReadyToSign</code>\nReturns <code>true</code> if the state in <code>RegisteredNotAbleToSign</code>\nReturns <code>true</code> if the state in <code>Unregistered</code>\nFile path to the KES secret key of the pool\nmetrics module. This module contains the signer metrics …\nMetrics HTTP Server IP.\nMetrics HTTP server IP.\nMetrics HTTP Server listening port.\nMetrics HTTP server listening port.\nCardano network\nCardano Network Magic number useful for TestNet &amp; DevNet\nAlso known as <code>k</code>, it defines the number of blocks that are …\nNetwork security parameter\nCreate a new Runner instance.\nCreate a new StateMachine instance.\nFile path to the operational certificate of the pool\nParty Id\nBlocks offset, from the tip of the chain, to exclude …\nPreload security parameter\nPreloading refresh interval in seconds\nRegister the signer verification key to the aggregator.\nRelay endpoint\nIf set the existing immutables digests cache will be reset.\nLaunch the state machine until an error occurs or it is …\nRun Interval\nServices\nAlternative storage backends when relational database …\nStore retention limit. If set to None, no limit will be …\nChunk size for importing transactions, combined with …\nChunk size for importing transactions\nMethod to convert.\nMethod to trigger the conversion.\nRead the current era and update the EraChecker.\nRead the stake distribution and store it.\nPerform the upkeep tasks.\nWrite the error to the given logger.\nContext error message\nContext error message\nEventual previous error message\nEventual previous error message\nCurrent Epoch\nEpoch when signer transitioned to the state.\nEpoch when signer transitioned to the state.\nMigration module\nSigner related database records\nSigner related database repositories\nGet all the migrations required by this version of the …\nDatabase record of a beacon signed by the signer\nThe epoch when the beacon was issued\nReturns the argument unchanged.\nDatetime when the beacon was initiated\nCalls <code>U::from(self)</code>.\nDatetime when the beacon was signed\nThe signed entity type to sign\nA SignedBeaconStore implementation using SQLite.\nReturns the argument unchanged.\nGet the last signed beacon.\nCalls <code>U::from(self)</code>.\nCreate a new instance of the <code>SignedBeaconRepository</code>.\nPrune all signed beacons that have an epoch below the …\nThe <code>DependenciesBuilder</code> is intended to manage Services …\nEpochServiceWrapper wraps a EpochService\nThis structure groups all the dependencies required by the …\nAPI version provider\nBuild dependencies for the Production environment.\nBuild a SQLite connection.\nCardano transactions preloader\nCertificate handler service\nCertifier service\nChain Observer service\nDigester service\nEpoch service\nEra checker service\nEra reader service\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nMetrics service\nCreate a new <code>DependenciesBuilder</code>.\nOverride default chain observer builder.\nOverride immutable file observer builder.\nProtocolInitializer store\nReturn a copy of the root logger.\nSignable Builder Service\nSigned entity type lock\nSingleSigner service\nStake store service\nTime point provider service\nUpkeep service\nBeacon to sign\nSignerEpochSettings represents the settings of an epoch\nCardano transactions signing configuration for the current …\nCurrent Signers\nThe epoch when the beacon was issued\nCurrent Epoch\nReturns the argument unchanged.\nReturns the argument unchanged.\nDatetime when the beacon was initiated\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreate a new <code>BeaconToSign</code>\nCardano transactions signing configuration for the next …\nSigners that will be able to sign on the next epoch\nRegistration protocol parameters\nThe signed entity type to sign\nMetrics service which is responsible for recording and …\nReturns the argument unchanged.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nGet the <code>$metric_attribute</code> counter.\nCalls <code>U::from(self)</code>.\nCreate a new MetricsService instance.\nAdapter error\nAvk computation Error\nTrait for mocking and testing a <code>AggregatorClient</code>\nError structure for the Aggregator Client.\nAggregatorHTTPClient is a http client for an aggregator\nIncompatible API version error\nImport and store CardanoTransaction.\nCardanoTransactionsPreloaderActivationSigner\nCertifier Service\nDefine the task responsible for pruning a datasource below …\nService that aggregates all data that don’t change in a …\nErrors dedicated to the EpochService.\nHTTP client creation error\nTrait to get the highest transaction block number\nMostly network errors.\nCould not parse response.\nImplementation of the epoch service.\nThis is responsible for creating new instances of …\nImplementation of the SingleSigner.\nRaised when service has not collected data at least once.\nCryptographic Signer creation error.\nProxy creation error\nThe aggregator host responded it cannot fulfill our …\nThe aggregator host has returned a technical error.\nCould not reach aggregator.\nSignature Error\nPublishes computed single signatures to a third party.\nTrait to store beacons that have been signed in order to …\nTrait to provide the current signed entity configuration …\nImplementation of the Certifier Service for the Mithril …\nSignableSeedBuilder signer implementation\nSimple wrapper to the EpochService to implement the …\nImplementation of the upkeep service for the signer.\nThe SingleSigner is the structure responsible for issuing …\nSingleSigner error structure.\nCardano transactions pruner\nCardano transactions store\nA decorator of TransactionsImporter that does the import …\nA decorator of TransactionsImporter that prunes the …\nA decorator of TransactionsImporter that vacuums the …\nUnhandled status code\nDefine the service responsible for the upkeep of the …\nGet the list of signed entity types that are allowed to …\nCreate a ProtocolInitializer instance.\nCheck if the given signer can sign for the current epoch\nGet the cardano transactions signing configuration for the …\nCompute and publish a single signature for a given …\nComputes single signatures\nGet signers for the current epoch\nGet signers with stake for the current epoch\nGet the current epoch for which the data stored in this …\nFilter out already signed entities from a list of signed …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreate an <code>AggregatorClientError</code> from a response.\nGet the highest known transaction block number\nGet the current signed entity configuration.\nGet the beacon to sign.\nGet the highest known transaction beacon\nGet the highest stored block range root bounds\nGet party id\nGet party id\nGet transactions in an interval of blocks\nInform the service a new epoch has been detected, telling …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nMark a beacon as signed.\nAggregatorHTTPClient factory\nCreate a new instance of <code>TransactionsImporterByChunk</code>.\nCreate a new instance of TransactionsImporterWithPruner.\nCreate a new instance of TransactionsImporterWithVacuum.\nCreate a new instance of …\nCreate a new <code>SignerCertifierService</code> instance.\nCreate a new service instance\nCreate a new instance of the …\nSignerSignableSeedBuilder factory\nCreate a new instance of the MithrilSingleSigner.\nCreate a new instance of the aggregator upkeep service.\nConstructor\nGet the cardano transactions signing configuration for the …\nGet signers for the next epoch\nGet signers with stake for the next epoch\nForge a client request adding protocol version in the …\nGet the protocol initializer for the current epoch if any\nPrune the transactions older than the given number of …\nPrune the datasource based on the given current epoch.\nGet the name of the data that will be pruned.\nPublish computed single signatures.\nRegisters single signatures with the aggregator.\nRegisters signer with the aggregator.\nGet protocol parameters for registration.\nRemove transactions and block range roots that are in a …\nRetrieves aggregator features message from the aggregator\nRetrieves epoch settings from the aggregator\nRun the upkeep service.\nStore list of block ranges with their corresponding merkle …\nStore list of transactions\nA Merkle tree store with Sqlite backend\nImplementation of the ProtocolInitializerStorer\nStore the ProtocolInitializer used for each Epoch. This is …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturn the list of the N last saved protocol initializers …\nFetch a protocol initializer if any saved for the given …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreate a new ProtocolInitializerStore.\nSave a protocol initializer for the given Epoch.")