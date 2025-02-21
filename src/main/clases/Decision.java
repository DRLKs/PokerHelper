package main.clases;
/**
 * 
 * @author DRLK 
 */
public class Decision {

	/**
	 * FOLD = 0   <br> 
	 * CHECK = 1  <br>
	 * CALL = 2   <br>
	 * RAISE = 3  <br>
	 * BET = 4    <br>
	 * ALL_IN = 5 <br>
	 */
	private int decision;
	
	static final int FOLD = 0;
	static final int CHECK = 1;
	static final int CALL = 2;
	static final int RAISE = 3;
	static final int BET = 4;
	static final int ALL_IN = 5;
	
	/**
	 * Apuesta que debería realizar el jugador
	 */
	private int aumentoApuesta;
	
	/**
	 * Inicializamos la decisión <br>
	 * Inicializado a <b>FOLD</b>
	 */
	public Decision() {
		setDecision(FOLD);
	}
	
	public int getCodigoDecision() {
		return decision;
	}
	
	void setDecision(int decision) {
		if( decision == FOLD ) {
			setAumentoApuesta(0);
		}
		this.decision = decision;
	}

	public int getAumentoApuesta() {
		return aumentoApuesta;
	}

	void setAumentoApuesta(int aumentoApuesta) {
		this.aumentoApuesta = aumentoApuesta;
	}

	@Override
	public String toString() {
		String decisionString;
		switch (decision) {
		case 0: 
			decisionString = "FOLD";
			break;
		case 1:
			decisionString = "CHECK, pasa";
			break;
		case 2:
			decisionString = "CALL, iguala añade " + aumentoApuesta + " al bote";
			break;
		case 3:
			decisionString = "RAISE, aumenta la apuesta en " + aumentoApuesta;
			break;
		case 4:
			decisionString = "BET , aumenta la apuesta en " + aumentoApuesta;
			break;
		case 5:
			decisionString = "ALL_IN, ve con todo";
			break;
		default:
			throw new IllegalArgumentException("Unexpected value: " + decision);
		}
		return decisionString;
	}
	
}