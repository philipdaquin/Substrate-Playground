//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
//* The service is the part of the node that coordinates communication between all other parts */

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
#![allow(clippy::needless_borrow)]
use node_template_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::{ExecutorProvider, RemoteBackend};
use sc_executor::NativeElseWasmExecutor;

use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, TaskManager};
use pow::*;
use sp_api::TransactionFor;
use sp_consensus::import_queue::BasicQueue;
use sp_core::{Encode, U256};
pub use sc_basic_authorship::*;
use std::thread;
use std::{sync::Arc, time::Duration};
pub use sc_executor::NativeElseWasmExecutor;

// Our native executor instance.
pub struct ExecutorDispatch;

//	Out native executor instance 
impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	/// Only enable the benchmarking host functions when we actually want to benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	/// Otherwise we only use the default Substrate host functions.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		node_template_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		node_template_runtime::native_version()
	}
}

type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

//* --- Inherent Data Providers ---- */
pub fn build_inherent_data_providers() -> Result<InherentDataProviders, ServiceError> {
    let providers = InherentDataProviders::new();

    providers
        .register_provider(sp_timestamp::InherentDataProvider)
        .map_err(Into::into)
        .map_err(sp_consensus::error::Error::InherentData)?;

    Ok(providers)
}
//	Anything that implements the ProvideInherentData trait\
//	The block authoring logic must supply all inherents that the runtime expects 

//* GRANDPA  */
// pub fn new_partial(
// 	config: &Configuration,
// ) -> Result<
// 	sc_service::PartialComponents<
// 		FullClient,
// 		FullBackend,
// 		FullSelectChain,
// 		sc_consensus::DefaultImportQueue<Block, FullClient>,
// 		sc_transaction_pool::FullPool<Block, FullClient>,
// 		(
// 			sc_finality_grandpa::GrandpaBlockImport<
// 				FullBackend,
// 				Block,
// 				FullClient,
// 				FullSelectChain,
// 			>,
// 			sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
// 			Option<Telemetry>,
// 		),
// 	>,
// 	ServiceError,
// > {
// 	if config.keystore_remote.is_some() {
// 		return Err(ServiceError::Other(format!("Remote Keystores are not supported.")))
// 	}

// 	let telemetry = config
// 		.telemetry_endpoints
// 		.clone()
// 		.filter(|x| !x.is_empty())
// 		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
// 			let worker = TelemetryWorker::new(16)?;
// 			let telemetry = worker.handle().new_telemetry(endpoints);
// 			Ok((worker, telemetry))
// 		})
// 		.transpose()?;

// 	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
// 		config.wasm_method,
// 		config.default_heap_pages,
// 		config.max_runtime_instances,
// 	);

// 	let (client, backend, keystore_container, task_manager) =
// 		sc_service::new_full_parts::<Block, RuntimeApi, _>(
// 			&config,
// 			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
// 			executor,
// 		)?;
// 	let client = Arc::new(client);

// 	let telemetry = telemetry.map(|(worker, telemetry)| {
// 		task_manager.spawn_handle().spawn("telemetry", worker.run());
// 		telemetry
// 	});

// 	let select_chain = sc_consensus::LongestChain::new(backend.clone());

// 	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
// 		config.transaction_pool.clone(),
// 		config.role.is_authority().into(),
// 		config.prometheus_registry(),
// 		task_manager.spawn_essential_handle(),
// 		client.clone(),
// 	);

// 	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
// 		client.clone(),
// 		&(client.clone() as Arc<_>),
// 		select_chain.clone(),
// 		telemetry.as_ref().map(|x| x.handle()),
// 	)?;

// 	let slot_duration = sc_consensus_aura::slot_duration(&*client)?.slot_duration();

// 	let import_queue =
// 		sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
// 			block_import: grandpa_block_import.clone(),
// 			justification_import: Some(Box::new(grandpa_block_import.clone())),
// 			client: client.clone(),
// 			create_inherent_data_providers: move |_, ()| async move {
// 				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

// 				let slot =
// 					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
// 						*timestamp,
// 						slot_duration,
// 					);

// 				Ok((timestamp, slot))
// 			},
// 			spawner: &task_manager.spawn_essential_handle(),
// 			can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
// 				client.executor().clone(),
// 			),
// 			registry: config.prometheus_registry(),
// 			check_for_equivocation: Default::default(),
// 			telemetry: telemetry.as_ref().map(|x| x.handle()),
// 		})?;

// 	Ok(sc_service::PartialComponents {
// 		client,
// 		backend,
// 		task_manager,
// 		import_queue,
// 		keystore_container,
// 		select_chain,
// 		transaction_pool,
// 		other: (grandpa_block_import, grandpa_link, telemetry),
// 	})
// }

//* Proof of Work */
pub fn new_partial(
	config: &Configuration
) -> Result<
	PartialComponents<
		FullClient, 
		FullBackend, 
		FullSelectChain, 
		BasicQueue<Block, TransactionFor<FullClient, Block>>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		sc_consensus_pow::PowBlockImport<
			Block, 
			Arc<FullClient>,
			FullClient,
			FullSelectChain,
			SimplePoWAlgo,
			impl sp_consensus::CanAuthorWith<Block>,
		>,
	>,
	ServiceError
>
{ 
	let inherent_data_providers = build_inherent_data_providers()?;

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

	//* ImportQueue, we provide references to our client */
	let pow_block_import = sc_consensus_pow::PowBlockImport::new(
		client.clone(),
		client.clone(),
		sha3pow::MinimalSha3Algorithm,
		0, // check inherents starting at block 0
		select_chain.clone(),
		inherent_data_providers.clone(),
		can_author_with,
	);

	let import_queue = sc_consensus_pow::import_queue(
		Box::new(pow_block_import.clone()),
		None,
		sha3pow::MinimalSha3Algorithm,
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	Ok(PartialComponents {
		client,
		backend,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain,
		other: pow_block_import,
	})
}


// fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
// 	// FIXME: here would the concrete keystore be built,
// 	//        must return a concrete type (NOT `LocalKeystore`) that
// 	//        implements `CryptoStore` and `SyncCryptoStore`
// 	Err("Remote Keystore not supported.")
// }

/// Builds a new service for a full client.
pub fn new_full(mut config: Configuration) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		mut keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, grandpa_link, mut telemetry),
	} = new_partial(&config)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			warp_sync,
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let is_authority = config.role.is_authority();
	let prometheus_registry = config.prometheus_registry().cloned();

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder: Box::new(|_, _| ()),
		on_demand: None,
		remote_blockchain: None,
		backend,
		telemetry,
		//network_status_sinks,
		system_rpc_tx,
		config,
	})?;

	if is_authority {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
		);

		let can_author_with =
			sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

		// Parameter details:
		//   https://substrate.dev/rustdocs/v3.0.0/sc_consensus_pow/fn.start_mining_worker.html
		// Also refer to kulupu config:
		//   https://github.com/kulupu/kulupu/blob/master/src/service.rs
		let (_worker, worker_task) = sc_consensus_pow::start_mining_worker(
			Box::new(pow_block_import),
			client,
			select_chain,
			MinimalSha3Algorithm,
			proposer,
			network,
			None,
			inherent_data_providers,
			// time to wait for a new block before starting to mine a new one
			Duration::from_secs(10),
			// how long to take to actually build the block (i.e. executing extrinsics)
			Duration::from_secs(10),
			can_author_with,
		);

		task_manager
			.spawn_essential_handle()
			.spawn_blocking("pow", worker_task);

		// Start Mining
		let mut nonce: U256 = U256::from(0);
		thread::spawn(move || loop {
			let worker = _worker.clone();
			let metadata = worker.lock().metadata();
			if let Some(metadata) = metadata {
				let compute = Compute {
					difficulty: metadata.difficulty,
					pre_hash: metadata.pre_hash,
					nonce,
				};
				let seal = compute.compute();
				if hash_meets_difficulty(&seal.work, seal.difficulty) {
					nonce = U256::from(0);
					let mut worker = worker.lock();
					worker.submit(seal.encode());
				} else {
					nonce = nonce.saturating_add(U256::from(1));
					if nonce == U256::MAX {
						nonce = U256::from(0);
					}
				}
			} else {
				thread::sleep(Duration::new(1, 0));
			}
		});
	}

	network_starter.start_network();
	Ok(task_manager)
}


/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
	let (client, backend, keystore_container, mut task_manager, on_demand) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

	let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
		on_demand.clone(),
	));

	let select_chain = sc_consensus::LongestChain::new(backend.clone());
	let inherent_data_providers = build_inherent_data_providers()?;
	// FixMe #375
	let _can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

	let pow_block_import = sc_consensus_pow::PowBlockImport::new(
		client.clone(),
		client.clone(),
		sha3pow::MinimalSha3Algorithm,
		0, // check inherents starting at block 0
		select_chain,
		inherent_data_providers.clone(),
		// FixMe #375
		sp_consensus::AlwaysCanAuthor,
	);

	let import_queue = sc_consensus_pow::import_queue(
		Box::new(pow_block_import),
		None,
		sha3pow::MinimalSha3Algorithm,
		inherent_data_providers,
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: Some(on_demand.clone()),
			warp_sync,
			block_announce_validator_builder: None,
		})?;

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		remote_blockchain: Some(backend.remote_blockchain()),
		transaction_pool,
		task_manager: &mut task_manager,
		on_demand: Some(on_demand),
		rpc_extensions_builder: Box::new(|_, _| ()),
		config,
		client,
		keystore: keystore_container.sync_keystore(),
		backend,
		network,
		telemetry,
		system_rpc_tx,
	})?;

	network_starter.start_network();

	Ok(task_manager)
}
	// if let Some(url) = &config.keystore_remote {
	// 	match remote_keystore(url) {
	// 		Ok(k) => keystore_container.set_remote_keystore(k),
	// 		Err(e) =>
	// 			return Err(ServiceError::Other(format!(
	// 				"Error hooking up remote keystore for {}: {}",
	// 				url, e
	// 			))),
	// 	};
	// }

	// config.network.extra_sets.push(sc_finality_grandpa::grandpa_peers_set_config());
	// let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
	// 	backend.clone(),
	// 	grandpa_link.shared_authority_set().clone(),
	// 	Vec::default(),
	// ));

	// let (network, system_rpc_tx, network_starter) =
	// 	sc_service::build_network(sc_service::BuildNetworkParams {
	// 		config: &config,
	// 		client: client.clone(),
	// 		transaction_pool: transaction_pool.clone(),
	// 		spawn_handle: task_manager.spawn_handle(),
	// 		import_queue,
	// 		on_demand: None,
	// 		block_announce_validator_builder: None,
	// 		warp_sync: Some(warp_sync),
	// 	})?;

	// if config.offchain_worker.enabled {
	// 	sc_service::build_offchain_workers(
	// 		&config,
	// 		task_manager.spawn_handle(),
	// 		client.clone(),
	// 		network.clone(),
	// 	);
	// }

	// let role = config.role.clone();
	// let force_authoring = config.force_authoring;
	// let backoff_authoring_blocks: Option<()> = None;
	// let name = config.network.node_name.clone();
	// let enable_grandpa = !config.disable_grandpa;
	// let prometheus_registry = config.prometheus_registry().cloned();

	// let rpc_extensions_builder = {
	// 	let client = client.clone();
	// 	let pool = transaction_pool.clone();

	// 	Box::new(move |deny_unsafe, _| {
	// 		let deps =
	// 			crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };

	// 		Ok(crate::rpc::create_full(deps))
	// 	})
	// };

	// let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
	// 	network: network.clone(),
	// 	client: client.clone(),
	// 	keystore: keystore_container.sync_keystore(),
	// 	task_manager: &mut task_manager,
	// 	transaction_pool: transaction_pool.clone(),
	// 	rpc_extensions_builder,
	// 	on_demand: None,
	// 	remote_blockchain: None,
	// 	backend,
	// 	system_rpc_tx,
	// 	config,
	// 	telemetry: telemetry.as_mut(),
	// })?;

	// if role.is_authority() {
	// 	let proposer_factory = sc_basic_authorship::ProposerFactory::new(
	// 		task_manager.spawn_handle(),
	// 		client.clone(),
	// 		transaction_pool,
	// 		prometheus_registry.as_ref(),
	// 		telemetry.as_ref().map(|x| x.handle()),
	// 	);

	// 	let can_author_with =
	// 		sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

	// 	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
	// 	let raw_slot_duration = slot_duration.slot_duration();

	// 	let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _, _>(
	// 		StartAuraParams {
	// 			slot_duration,
	// 			client: client.clone(),
	// 			select_chain,
	// 			block_import,
	// 			proposer_factory,
	// 			create_inherent_data_providers: move |_, ()| async move {
	// 				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

	// 				let slot =
	// 					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
	// 						*timestamp,
	// 						raw_slot_duration,
	// 					);

	// 				Ok((timestamp, slot))
	// 			},
	// 			force_authoring,
	// 			backoff_authoring_blocks,
	// 			keystore: keystore_container.sync_keystore(),
	// 			can_author_with,
	// 			sync_oracle: network.clone(),
	// 			justification_sync_link: network.clone(),
	// 			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
	// 			max_block_proposal_slot_portion: None,
	// 			telemetry: telemetry.as_ref().map(|x| x.handle()),
	// 		},
	// 	)?;

	// 	// the AURA authoring task is considered essential, i.e. if it
	// 	// fails we take down the service with it.
	// 	task_manager.spawn_essential_handle().spawn_blocking("aura", aura);
	//}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	// let keystore =
	// 	if role.is_authority() { Some(keystore_container.sync_keystore()) } else { None };

	// let grandpa_config = sc_finality_grandpa::Config {
	// 	// FIXME #1578 make this available through chainspec
	// 	gossip_duration: Duration::from_millis(333),
	// 	justification_period: 512,
	// 	name: Some(name),
	// 	observer_enabled: false,
	// 	keystore,
	// 	local_role: role,
	// 	telemetry: telemetry.as_ref().map(|x| x.handle()),
	// };

	// if enable_grandpa {
	// 	// start the full GRANDPA voter
	// 	// NOTE: non-authorities could run the GRANDPA observer protocol, but at
	// 	// this point the full voter should provide better guarantees of block
	// 	// and vote data availability than the observer. The observer has not
	// 	// been tested extensively yet and having most nodes in a network run it
	// 	// could lead to finality stalls.
	// 	let grandpa_config = sc_finality_grandpa::GrandpaParams {
	// 		config: grandpa_config,
	// 		link: grandpa_link,
	// 		network,
	// 		voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
	// 		prometheus_registry,
	// 		shared_voter_state: SharedVoterState::empty(),
	// 		telemetry: telemetry.as_ref().map(|x| x.handle()),
	// 	};

	// 	// the GRANDPA voter task is considered infallible, i.e.
	// 	// if it fails we take down the service with it.
	// 	task_manager.spawn_essential_handle().spawn_blocking(
	// 		"grandpa-voter",
	// 		sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
	// 	);
	// }

	// network_starter.start_network();
// 	Ok(task_manager)
// }
