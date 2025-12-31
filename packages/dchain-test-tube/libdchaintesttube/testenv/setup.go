package testenv

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	// helpers

	// tendermint
	"cosmossdk.io/errors"
	log "cosmossdk.io/log"
	abci "github.com/cometbft/cometbft/abci/types"
	dbm "github.com/cosmos/cosmos-db"

	// cosmos-sdk
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/server"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktestutil "github.com/cosmos/cosmos-sdk/x/bank/testutil"
	slashingtypes "github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	// wasmd
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"

	// dchain
	"github.com/d-foundation/protocol/app"
	dappparams "github.com/d-foundation/protocol/app/params"

	sdkmath "cosmossdk.io/math"

	tmtypes "github.com/cometbft/cometbft/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"

	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"

	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	depository "github.com/d-foundation/protocol/x/depository"
	depositorytypes "github.com/d-foundation/protocol/x/depository/types"

	notary "github.com/d-foundation/protocol/x/notary"
	notarytypes "github.com/d-foundation/protocol/x/notary/types"
)

func GenesisStateWithValSet(appInstance *app.DChainApp) (map[string]json.RawMessage, secp256k1.PrivKey) {
	privVal := NewPV()
	pubKey, _ := privVal.GetPubKey()
	validator := tmtypes.NewValidator(pubKey, 1)
	valSet := tmtypes.NewValidatorSet([]*tmtypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	senderPrivKey.PubKey().Address()
	acc := authtypes.NewBaseAccountWithAddress(senderPrivKey.PubKey().Address().Bytes())

	//////////////////////
	balances := []banktypes.Balance{}

	genesisState := appInstance.DefaultGenesis()
	genAccs := []authtypes.GenesisAccount{acc}
	authGenesis := authtypes.NewGenesisState(authtypes.DefaultParams(), genAccs)
	genesisState[authtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(authGenesis)

	validators := make([]stakingtypes.Validator, 0, len(valSet.Validators))
	delegations := make([]stakingtypes.Delegation, 0, len(valSet.Validators))

	bondAmt := sdk.DefaultPowerReduction
	initValPowers := []abci.ValidatorUpdate{}

	for _, val := range valSet.Validators {
		pk, _ := cryptocodec.FromCmtPubKeyInterface(val.PubKey)
		pkAny, _ := codectypes.NewAnyWithValue(pk)
		validator := stakingtypes.Validator{
			OperatorAddress:   sdk.ValAddress(val.Address).String(),
			ConsensusPubkey:   pkAny,
			Jailed:            false,
			Status:            stakingtypes.Bonded,
			Tokens:            bondAmt,
			DelegatorShares:   sdkmath.LegacyOneDec(),
			Description:       stakingtypes.Description{},
			UnbondingHeight:   int64(0),
			UnbondingTime:     time.Unix(0, 0).UTC(),
			Commission:        stakingtypes.NewCommission(sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec()),
			MinSelfDelegation: sdkmath.ZeroInt(),
		}

		valAddr, err := sdk.ValAddressFromHex(val.Address.String())
		requireNoErr(err)
		validators = append(validators, validator)
		delegations = append(delegations, stakingtypes.NewDelegation(genAccs[0].GetAddress().String(), valAddr.String(), sdkmath.LegacyOneDec()))

		// add initial validator powers so consumer InitGenesis runs correctly
		pub, _ := val.ToProto()
		initValPowers = append(initValPowers, abci.ValidatorUpdate{
			Power:  val.VotingPower,
			PubKey: pub.PubKey,
		})
	}
	// set validators and delegations
	stakingGenesis := stakingtypes.NewGenesisState(stakingtypes.DefaultParams(), validators, delegations)
	genesisState[stakingtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(stakingGenesis)

	totalSupply := sdk.NewCoins()
	for _, b := range balances {
		// add genesis acc tokens to total supply
		totalSupply = totalSupply.Add(b.Coins...)
	}

	for range delegations {
		// add delegated tokens to total supply
		totalSupply = totalSupply.Add(sdk.NewCoin(dappparams.DefaultBondDenom, bondAmt))
	}

	// add bonded amount to bonded pool module account
	balances = append(balances, banktypes.Balance{
		Address: authtypes.NewModuleAddress(stakingtypes.BondedPoolName).String(),
		Coins:   sdk.Coins{sdk.NewCoin(dappparams.DefaultBondDenom, bondAmt)},
	})

	// create denom metadata for udt (required for cosmos-sdk v0.53.4+ collections-based bank module)
	denomMetadata := []banktypes.Metadata{
		{
			Description: "The native staking token of the DChain network.",
			DenomUnits: []*banktypes.DenomUnit{
				{
					Denom:    dappparams.DefaultBondDenom,
					Exponent: 0,
					Aliases:  []string{""},
				},
			},
			Base:    dappparams.DefaultBondDenom,
			Display: "udt",
		},
	}

	// update total supply
	bankGenesis := banktypes.NewGenesisState(
		banktypes.DefaultGenesisState().Params,
		balances,
		totalSupply,
		denomMetadata,
		[]banktypes.SendEnabled{},
	)
	genesisState[banktypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(bankGenesis)

	_, err := tmtypes.PB2TM.ValidatorUpdates(initValPowers)
	if err != nil {
		panic("failed to get vals")
	}

	return genesisState, secp256k1.PrivKey{Key: privVal.PrivKey.Bytes()}
}

// TestEnv for DChain - NOTE: No ParamTypesRegistry
type TestEnv struct {
	App      *app.DChainApp
	Ctx      sdk.Context
	ValPrivs []*secp256k1.PrivKey
	NodeHome string
}

// TestAppOptions is a stub implementing AppOptions
type TestAppOptions struct {
	NodeHome string
}

// Get implements AppOptions
func (ao TestAppOptions) Get(o string) interface{} {
	if o == server.FlagTrace {
		return true
	}

	if o == "wasm.simulation_gas_limit" {
		return ^uint64(0) // max uint64
	}

	if o == flags.FlagHome {
		return ao.NodeHome
	}

	return nil
}

func NewDChainApp(nodeHome string) *app.DChainApp {
	db := dbm.NewMemDB()

	return app.NewDChainApp(
		log.NewNopLogger(),
		db,
		nil,
		true,
		TestAppOptions{NodeHome: nodeHome},
		[]wasmkeeper.Option{},
		baseapp.SetChainID("dchain-1"),
	)
}

func InitChain(appInstance *app.DChainApp, platformAdmin string) (sdk.Context, secp256k1.PrivKey) {
	sdk.DefaultBondDenom = "udt"
	genesisState, valPriv := GenesisStateWithValSet(appInstance)

	// Set up Wasm genesis state
	wasmGen := wasmtypes.GenesisState{
		Params: wasmtypes.Params{
			// Allow store code without gov
			CodeUploadAccess:             wasmtypes.AllowEverybody,
			InstantiateDefaultPermission: wasmtypes.AccessTypeEverybody,
		},
	}
	genesisState[wasmtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(&wasmGen)

	// Set up depository genesis state with PlatformAdmin if provided
	if platformAdmin != "" {
		depositoryParams, err := depositorytypes.NewParams(platformAdmin)
		requireNoErr(err)

		depositoryGen := depositorytypes.GenesisState{
			Params:            depositoryParams,
			Depositories:      []*depositorytypes.Depository{},
			GlobalNotes:       []*depositorytypes.GlobalNote{},
			DepositoryCounter: 0,
		}
		genesisState[depository.ModuleName] = appInstance.AppCodec().MustMarshalJSON(&depositoryGen)
	}

	// Set up notary genesis state
	notaryGen := notarytypes.GenesisState{
		NextNotaryInfoId: 1,
		AssetTypeMap:     map[uint64]string{1: "invoice"},
		CurrencyConversionRates: map[int32]notarytypes.ConversionRate{
			1: notarytypes.ConversionRate{ConversiontRate: sdkmath.LegacyNewDec(1)},
			2: notarytypes.ConversionRate{ConversiontRate: sdkmath.LegacyNewDecWithPrec(85, 2)},
			3: notarytypes.ConversionRate{ConversiontRate: sdkmath.LegacyNewDecWithPrec(87, 2)},
		},
		EurPriceInUdt:       sdkmath.LegacyNewDecWithPrec(250000000000000000, 9), // 250_000_000.000000000
		NotarisationFeeRate: sdkmath.LegacyNewDecWithPrec(1, 2),                  // 0.01
	}
	genesisState[notary.ModuleName] = appInstance.AppCodec().MustMarshalJSON(&notaryGen)

	// set staking genesis state
	stakingGenesisState := stakingtypes.GenesisState{}
	appInstance.AppCodec().UnmarshalJSON(genesisState[stakingtypes.ModuleName], &stakingGenesisState)

	stateBytes, err := json.MarshalIndent(genesisState, "", " ")

	requireNoErr(err)

	concensusParams := simtestutil.DefaultConsensusParams
	concensusParams.Block = &cmtproto.BlockParams{
		MaxBytes: 22020096,
		MaxGas:   -1,
	}

	// replace sdk.DefaultDenom with "udt"
	stateBytes = []byte(strings.Replace(string(stateBytes), "\"stake\"", "\"udt\"", -1))

	appInstance.InitChain(
		&abci.RequestInitChain{
			Validators:      []abci.ValidatorUpdate{},
			ConsensusParams: concensusParams,
			AppStateBytes:   stateBytes,
			ChainId:         "dchain-1",
			Time:            time.Now().UTC(),
		},
	)

	ctx := appInstance.NewContextLegacy(false, cmtproto.Header{Height: 0, ChainID: "dchain-1", Time: time.Now().UTC()})

	// Manually set validator signing info, otherwise we panic
	vals, err := appInstance.StakingKeeper.GetAllValidators(ctx)
	if err != nil {
		panic(err)
	}

	for _, val := range vals {
		consAddr, _ := val.GetConsAddr()
		signingInfo := slashingtypes.NewValidatorSigningInfo(
			consAddr,
			ctx.BlockHeight(),
			0,
			time.Unix(0, 0),
			false,
			0,
		)
		err := appInstance.SlashingKeeper.SetValidatorSigningInfo(ctx, consAddr, signingInfo)
		if err != nil {
			panic(err)
		}
	}
	return ctx, valPriv
}

func (env *TestEnv) BeginNewBlock(executeNextEpoch bool, timeIncreaseSeconds uint64) {
	validators, err := env.App.StakingKeeper.GetAllValidators(env.Ctx)
	requireNoErr(err)
	valAddr, err := validators[0].GetConsAddr()
	requireNoErr(err)

	env.beginNewBlockWithProposer(executeNextEpoch, valAddr, timeIncreaseSeconds)
}

func (env *TestEnv) FundValidators() {
	for _, valPriv := range env.ValPrivs {
		valAddr := sdk.AccAddress(valPriv.PubKey().Address())
		err := banktestutil.FundAccount(env.Ctx, env.App.BankKeeper, valAddr, sdk.NewCoins(sdk.NewInt64Coin("udt", 9223372036854775807)))
		if err != nil {
			panic(errors.Wrapf(err, "Failed to fund account"))
		}
	}
}

func (env *TestEnv) InitValidator() []byte {
	valPriv, valAddrFancy := env.setupValidator(stakingtypes.Bonded)
	validator, _ := env.App.StakingKeeper.GetValidator(env.Ctx, valAddrFancy)
	valAddr, _ := validator.GetConsAddr()

	env.ValPrivs = append(env.ValPrivs, valPriv)
	err := banktestutil.FundAccount(env.Ctx, env.App.BankKeeper, sdk.AccAddress(valAddr), sdk.NewCoins(sdk.NewInt64Coin("udt", 9223372036854775807)))
	if err != nil {
		panic(errors.Wrapf(err, "Failed to fund account"))
	}

	return valAddr
}

func (env *TestEnv) GetValidatorAddresses() []string {
	validators, err := env.App.StakingKeeper.GetAllValidators(env.Ctx)
	requireNoErr(err)
	var addresses []string
	for _, validator := range validators {
		addresses = append(addresses, validator.OperatorAddress)
	}

	return addresses
}

// beginNewBlockWithProposer begins a new block with a proposer.
func (env *TestEnv) beginNewBlockWithProposer(executeNextEpoch bool, proposer sdk.ValAddress, timeIncreaseSeconds uint64) {
	validator, err := env.App.StakingKeeper.GetValidator(env.Ctx, proposer)
	requireNoErr(err)

	valConsAddr, err := validator.GetConsAddr()
	requireNoErr(err)

	valAddr := valConsAddr

	// DChain may not have epochs like Osmosis, so we'll simplify this
	newBlockTime := env.Ctx.BlockTime().Add(time.Duration(timeIncreaseSeconds) * time.Second)
	if executeNextEpoch {
		// If DChain has epochs, adjust this accordingly
		newBlockTime = env.Ctx.BlockTime().Add(24 * time.Hour)
	}

	header := cmtproto.Header{Height: env.Ctx.BlockHeight() + 1, Time: newBlockTime}
	env.Ctx = env.Ctx.WithBlockTime(newBlockTime).WithBlockHeight(env.Ctx.BlockHeight() + 1)
	voteInfos := []abci.VoteInfo{{
		Validator:   abci.Validator{Address: valAddr, Power: 1000},
		BlockIdFlag: cmtproto.BlockIDFlagCommit,
	}}
	env.Ctx = env.Ctx.WithVoteInfos(voteInfos)

	_, err = env.App.BeginBlocker(env.Ctx)
	requireNoErr(err)

	env.Ctx = env.App.NewContextLegacy(false, header)
}

func (env *TestEnv) setupValidator(bondStatus stakingtypes.BondStatus) (*secp256k1.PrivKey, sdk.ValAddress) {
	valPriv := secp256k1.GenPrivKey()
	valPub := valPriv.PubKey()
	valAddr := sdk.ValAddress(valPub.Address())
	params, err := env.App.StakingKeeper.GetParams(env.Ctx)
	requireNoErr(err)
	bondDenom := params.BondDenom
	selfBond := sdk.NewCoins(sdk.Coin{Amount: sdkmath.NewInt(100), Denom: bondDenom})

	err = banktestutil.FundAccount(env.Ctx, env.App.BankKeeper, sdk.AccAddress(valPub.Address()), selfBond)
	requireNoErr(err)

	stakingMsgServer := stakingkeeper.NewMsgServerImpl(env.App.StakingKeeper)
	stakingCoin := sdk.NewCoin(bondDenom, selfBond[0].Amount)
	ZeroCommission := stakingtypes.NewCommissionRates(sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec())
	msg, err := stakingtypes.NewMsgCreateValidator(valAddr.String(), valPub, stakingCoin, stakingtypes.Description{}, ZeroCommission, sdkmath.OneInt())
	requireNoErr(err)

	res, err := stakingMsgServer.CreateValidator(env.Ctx, msg)
	requireNoErr(err)
	requireNoNil("staking handler", res)

	env.App.BankKeeper.SendCoinsFromModuleToModule(env.Ctx, stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, sdk.NewCoins(stakingCoin))

	val, err := env.App.StakingKeeper.GetValidator(env.Ctx, valAddr)
	requireNoErr(err)

	val = val.UpdateStatus(bondStatus)
	env.App.StakingKeeper.SetValidator(env.Ctx, val)

	consAddr, err := val.GetConsAddr()
	requireNoErr(err)
	env.setupDefaultValidatorSigningInfo(consAddr)

	return valPriv, valAddr
}

func (env *TestEnv) SetupDefaultValidator() {
	validators, err := env.App.StakingKeeper.GetAllValidators(env.Ctx)
	requireNoErr(err)
	valAddrFancy, err := validators[0].GetConsAddr()
	requireNoErr(err)
	env.setupDefaultValidatorSigningInfo(valAddrFancy)
}

func (env *TestEnv) setupDefaultValidatorSigningInfo(consAddr sdk.ConsAddress) {
	signingInfo := slashingtypes.NewValidatorSigningInfo(
		consAddr,
		env.Ctx.BlockHeight(),
		0,
		time.Unix(0, 0),
		false,
		0,
	)
	env.App.SlashingKeeper.SetValidatorSigningInfo(env.Ctx, consAddr, signingInfo)
}

// NOTE: SetupParamTypes is NOT needed for DChain - it doesn't use legacy params module

func requireNoErr(err error) {
	if err != nil {
		panic(err)
	}
}

func requireNoNil(name string, nilable any) {
	if nilable == nil {
		panic(fmt.Sprintf("%s must not be nil", name))
	}
}

func requierTrue(name string, b bool) {
	if !b {
		panic(fmt.Sprintf("%s must be true", name))
	}
}
